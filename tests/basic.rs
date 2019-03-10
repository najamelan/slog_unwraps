
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
		slog_unwraps      :: { *                                                                         } ,
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
		assert_eq!( &std::str::from_utf8( unlocked.deref() ).unwrap()[19..], " CRIT PANIC - fn `boom` calls `unwraps` @ tests/basic.rs:78 -> Error: No such file or directory (os error 2)\n" )
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
		assert_eq!( &std::str::from_utf8( unlocked.deref() ).unwrap()[19..], " CRIT PANIC - fn `{{closure}}` calls `unwraps` @ tests/basic.rs:101 -> Error: No such file or directory (os error 2)\n" )
	}
}
