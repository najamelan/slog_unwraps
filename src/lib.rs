//! Syntactic sugar to slog an error before panicking. It will add caller file and line information to the log statement,
//! but know that that only makes sense in debug mode. In release mode this information will either be missing or unreliable.
//!
//! ## Example
//!
//! ```rust should_panic
//! use
//! {
//!    std          :: { fs::File                       } ,
//!    slog         :: { Drain, Level, Logger, o        } ,
//!    slog_term    :: { FullFormat, PlainSyncDecorator } ,
//!    slog_unwraps :: { ResultExt                      } ,
//! };
//!
//! fn main()
//! {
//!    let plain = PlainSyncDecorator::new( std::io::stderr() )                  ;
//!    let log   = Logger::root( FullFormat::new( plain ).build().fuse(), o!() ) ;
//!
//!    let f = File::open( "dont.exist" );
//!    let g = File::open( "dont.exist" );
//!
//!    // This will output:
//!    //
//!    // Mar 08 18:13:52.034 CRIT PANIC - fn `main` calls `unwraps` @ examples/basic.rs:20 -> Error: No such file or directory (os error 2)
//!    // before panicking
//!    //
//!    f.unwraps( &log );
//!
//!    // This is equivalent. Of course you can do something else with the result after logging rather than unwrapping. This only logs
//!    // if the result is an error.
//!    //
//!    g.log( &log, Level::Critical ).unwrap();
//! }
//! ```
//!

use
{
	std::fmt  :: { Debug, Display                                       },
	backtrace :: { Backtrace                                            },
	regex     :: { Regex                                                },
	slog      :: { Logger, trace, debug, info, warn, error, crit, Level },
};


/// Add extras to the result type to ease logging of errors.
///
pub trait ResultExt<T, E>

	where E: Display + Debug
{
	/// Logs the error to the provided logger before unwrapping.
	///
	fn unwraps( self, log: &Logger ) -> T;

	/// logs a potential error in the result and returns the result intact.
	///
	fn log    ( self, log: &Logger, lvl: slog::Level ) -> Result<T,E>;
}


impl<T, E> ResultExt<T, E> for Result<T, E> where E: Display + Debug
{
	fn unwraps( self, log: &Logger ) -> T
	{
		self.map_err( |e|
		{
			crit!( log, "{} -> Error: {}" , demangle( "unwraps" ), e );
			e

		}).unwrap()
	}


	fn log( self, log: &Logger, lvl: Level ) -> Result<T, E>
	{
		self.map_err( |e|
		{
			match lvl
			{
				Level::Trace    => trace!( log, "{}", e ),
				Level::Debug    => debug!( log, "{}", e ),
				Level::Info     => info! ( log, "{}", e ),
				Level::Warning  => warn! ( log, "{}", e ),
				Level::Error    => error!( log, "{}", e ),
				Level::Critical => crit! ( log, "{}", e ),
			}

			e
		})
	}
}



// Demangle the API of the backtrace crate!
//
// Returns the caller function name + file:lineno for logging in ResultExtSlog
//
fn demangle( which: &str ) -> String
{
	let empty  = String::with_capacity(0);
	let bt     = Backtrace::new();
	let frames = bt.frames();

	let frame = &frames.get( 4 );

	if let Some( frame  ) = frame {
	if let Some( symbol ) = frame.symbols().last()
	{
		format!
		(
			  "PANIC - fn `{}` calls `{}` @ {}:{}"
			, symbol.name()    .map( |s| strip( format!( "{}", s ) )     ).unwrap_or_else( || empty.clone() )
			, which
			, symbol.filename().map( |s| s.to_string_lossy().to_string() ).unwrap_or_else( || empty.clone() )
			, symbol.lineno()  .map( |s| format!( "{}", s )              ).unwrap_or( empty )
		)

	} else { empty }
	} else { empty }
}



// Will return the function name from a string returned by backtrace:
//
// ekke::main::dkk39ru458u3 -> main
//
fn strip( input: String ) -> String
{
	let re = Regex::new( r"([^:]+)::[[:alnum:]]+$" ).unwrap();

	re.captures( &input )

		.map( |caps|

			caps.get(1)

				.map_or( String::new(), |m| m.as_str().to_string() )

		)

		.unwrap_or( input )
}



// It just makes no sense to run the tests in release mode... Debug symbols are stripped,
// so the output is compeletely different.
//
#[ cfg( debug_assertions ) ]
#[ cfg( test             ) ]
//
mod tests
{
	use
	{
		crate             :: { *                                                                         } ,
		std               :: { sync::Arc, sync::Mutex, io::Write, io::Error, ops::Deref, fs::File, panic } ,
		slog              :: { Drain, Logger, Level, o                                                   } ,
		slog_term         :: { FullFormat, PlainSyncDecorator                                            } ,
		pretty_assertions :: { assert_eq                                                                 } ,
	};


	struct LogWriter
	{
		pub buf: Arc<Mutex<Vec<u8>>>,
	}


	impl Write for LogWriter
	{
		fn write( &mut self, data: &[u8] ) -> Result< usize, Error >
		{
			self.buf.lock().unwrap().write( data )
		}

		fn flush( &mut self) -> Result<(), Error>
		{
			self.buf.lock().unwrap().flush()
		}
	}


	fn log( buf: LogWriter ) -> Logger
	{
		let plain = PlainSyncDecorator::new( buf );

		Logger::root( FullFormat::new( plain ).build().fuse(), o!() )
	}


	#[test]
	//
	fn test_log()
	{
		let buf = Arc::new( Mutex::new( Vec::new() ) );
		let log = log( LogWriter{ buf: buf.clone() } );

		let f = File::open( "dont.exist" );
		let _ = f.log( &log, Level::Error );

		let unlocked = buf.lock().unwrap();

		// Beginning of the string is the time information, which will change all the time
		//
		assert_eq!( &std::str::from_utf8( unlocked.deref() ).unwrap()[19..], " ERRO No such file or directory (os error 2)\n" )
	}


	#[test]
	//
	fn test_unwrap()
	{
		let buf = Arc::new( Mutex::new( Vec::new() ) );
		let log = log( LogWriter{ buf: buf.clone() } );

		let result = panic::catch_unwind(||
		{
			fn boom( log: &Logger )
			{
				let f = File::open( "dont.exist" );
				let _ = f.unwraps( log );
			}

			boom( &log );
		});

		let unlocked = buf.lock().unwrap();

		assert!( result.is_err() );
		assert_eq!( &std::str::from_utf8( unlocked.deref() ).unwrap()[19..], " CRIT PANIC - fn `boom` calls `unwraps` @ src/lib.rs:226 -> Error: No such file or directory (os error 2)\n" )
	}


	#[test]
	//
	fn test_unwrap_closure()
	{
		let buf = Arc::new( Mutex::new( Vec::new() ) );
		let log = log( LogWriter{ buf: buf.clone() } );

		let result = panic::catch_unwind(||
		{
			let f = File::open( "dont.exist" );
			let _ = f.unwraps( &log );
		});

		let unlocked = buf.lock().unwrap();

		assert!( result.is_err() );
		assert_eq!( &std::str::from_utf8( unlocked.deref() ).unwrap()[19..], " CRIT PANIC - fn `{{closure}}` calls `unwraps` @ src/lib.rs:249 -> Error: No such file or directory (os error 2)\n" )
	}
}
