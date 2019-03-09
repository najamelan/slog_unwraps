use
{
   std          :: { fs::File                       } ,
   slog         :: { Drain, Level, Logger, o, crit  } ,
   slog_term    :: { FullFormat, PlainSyncDecorator } ,
   slog_unwraps :: { ResultExt                      } ,
};

fn main()
{
   let plain = PlainSyncDecorator::new( std::io::stderr() )                  ;
   let log   = Logger::root( FullFormat::new( plain ).build().fuse(), o!() ) ;


   // This will output:
   //
   // Mar 08 18:13:52.034 CRIT PANIC - fn `main` calls `unwraps` @ examples/basic.rs:20 -> Error: No such file or directory (os error 2)
   //
   // and then will call unwrap for you
   //
   let f = File::open( "dont.exist" );
   let _file = f.unwraps( &log );


   // This is equivalent. Of course you can do something else with the result after logging rather than unwrapping. This only logs
   // if the result is an error.
   //
   let g = File::open( "dont.exist" );
   let _file = g.log( &log, Level::Critical ).unwrap();


   // Without this crate, everytime you want to unwrap, you would write something like:
   //
   let h = File::open( "dont.exist" );
   let _file = match h
   {
      Ok ( f ) => f,
      Err( e ) => { crit!( log, "{}", e ); panic!() }
   };
}
