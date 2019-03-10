//! [![docs](https://docs.rs/slog_unwraps/badge.svg)](https://docs.rs/slog_unwraps)
//! ![code size](https://img.shields.io/github/languages/code-size/najamelan/slog_unwraps.svg)
//! [![Build Status](https://api.travis-ci.org/najamelan/slog_unwraps.svg?branch=master)](https://travis-ci.org/najamelan/slog_unwraps)
//! [![Project Status: Active – The project has reached a stable, usable state and is being actively developed.](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)
//!
//! Syntactic sugar to slog an error before [unwrapping](https://doc.rust-lang.org/std/result/enum.Result.html#method.unwrap).
//! It will add caller file and line information to the log statement so you don't have to turn on RUST_BACKTRACE to see what
//! went wrong, but know that that only makes sense in debug mode. In release mode this information will either be missing or unreliable.
//!
//! At first I had an `expects` function as well to be able to add context, but I really think you should use the
//! [`failure` crate](https://docs.rs/failure), which provides a `context` method on errors, which is much cleaner, so `expects`
//! no longer exists. If you don't want to use `failure`, you will have to make sure your errors display sensible messages.
//!
//! ## Example
//!
//! run with `cargo run --example basic`
//!
//! ```rust should_panic
//! use
//! {
//!    std          :: { fs::File                       } ,
//!    slog         :: { Drain, Level, Logger, o, crit  } ,
//!    slog_term    :: { FullFormat, PlainSyncDecorator } ,
//!    slog_unwraps :: { ResultExt                      } ,
//! };
//!
//! fn main()
//! {
//!    let plain = PlainSyncDecorator::new( std::io::stderr() )                  ;
//!    let log   = Logger::root( FullFormat::new( plain ).build().fuse(), o!() ) ;
//!
//!
//!    // This will output (in one line, wrapped here for readablility):
//!    //
//!    // Mar 08 18:13:52.034 CRIT PANIC - fn `main` calls `unwraps` @ examples/basic.rs:20
//!    // -> Error: No such file or directory (os error 2)
//!    //
//!    // and then will call unwrap for you
//!    //
//!    let f     = File::open( "dont.exist" );
//!    let _file = f.unwraps( &log );
//!
//!
//!    // This is equivalent. Of course you can do something else with the result after logging
//!    // rather than unwrapping. This only logs if the result is an error.
//!    //
//!    let g     = File::open( "dont.exist" );
//!    let _file = g.log( &log, Level::Critical ).unwrap();
//!
//!    // Without this crate, everytime you want to unwrap, you would write something like:
//!    //
//!    let h     = File::open( "dont.exist" );
//!
//!    let _file = match h
//!    {
//!       Ok ( f ) => f,
//!       Err( e ) => { crit!( log, "{}", e ); panic!() }
//!    };
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


/// Extends the [std::result::Result](https://doc.rust-lang.org/std/result/enum.Result.html) type with extra methods to ease logging of errors.
///
pub trait ResultExt<T, E>

	where E: Display + Debug
{
	/// Logs the error to the provided logger before unwrapping.
	///
	fn unwraps( self, log: &Logger ) -> T;

	/// Logs a potential error in the result and returns the result intact.
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
