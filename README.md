# slog_unwraps

Syntactic sugar to slog an error before panicking. It will add caller file and line information to the log statement,
but know that that only makes sense in debug mode. In release mode this information will either be missing or unreliable.

### Example

```rust
use
{
   std          :: { fs::File                       } ,
   slog         :: { Drain, Level, Logger, o        } ,
   slog_term    :: { FullFormat, PlainSyncDecorator } ,
   slog_unwraps :: { ResultExt                      } ,
};

fn main()
{
   let plain = PlainSyncDecorator::new( std::io::stderr() )                  ;
   let log   = Logger::root( FullFormat::new( plain ).build().fuse(), o!() ) ;

   let f = File::open( "dont.exist" );
   let g = File::open( "dont.exist" );

   // This will output:
   //
   // Mar 08 18:13:52.034 CRIT PANIC - fn `main` calls `unwraps` @ examples/basic.rs:20 -> Error: No such file or directory (os error 2)
   // before panicking
   //
   f.unwraps( &log );

   // This is equivalent. Of course you can do something else with the result after logging rather than unwrapping. This only logs
   // if the result is an error.
   //
   g.log( &log, Level::Critical ).unwrap();
}
```rust

