
use
{
	std          :: { fs::File                } ,
	slog         :: { Drain, Level, Logger, o } ,
	slog_term    :: { FullFormat              } ,
	slog_unwraps :: { ResultExt               } ,
};

fn main()
{
	let plain = slog_term::PlainSyncDecorator::new( std::io::stderr() )       ;
	let log   = Logger::root( FullFormat::new( plain ).build().fuse(), o!()	) ;

	let f = File::open( "dont.exist" );
	let g = File::open( "dont.exist" );

	// This will output:
	//
	// Mar 08 18:13:52.034 CRIT PANIC - fn `main` calls `unwraps` @ examples/basic.rs:20 -> Error: No such file or directory (os error 2)
	// before panicking
	//
	f.unwraps( &log );

	// This is equivalent
	//
	g.log( &log, Level::Critical ).unwrap();
}
