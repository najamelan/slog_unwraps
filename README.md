# slog_unwraps [![docs](https://docs.rs/slog_unwraps/badge.svg)](https://docs.rs/slog_unwraps) ![code size](https://img.shields.io/github/languages/code-size/najamelan/slog_unwraps.svg) [![Build Status](https://api.travis-ci.org/najamelan/slog_unwraps.svg?branch=master)](https://travis-ci.org/najamelan/slog_unwraps) [![Project Status: Active – The project has reached a stable, usable state and is being actively developed.](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)

Syntactic sugar to slog an error before [unwrapping](https://doc.rust-lang.org/std/result/enum.Result.html#method.unwrap).
It will add caller file and line information to the log statement, but that only makes sense in debug mode.
In release mode this information will either be missing or unreliable.

Anyways, this is meant to make your life easier while developping. It will also report file and line number of the unwraps
so you don't have to turn on RUST_BACKTRACE to see what went wrong.

At first I had an `expects` function as well to be able to add context, but I really think you should use the
[`failure` crate](https://docs.rs/failure), which provides a `context` method on errors, which is cleaner, so `expects`
no longer exists. If you don't want to use `failure`, you will have to make sure your errors display sensible messages.

### Example

run with `cargo run --example basic`

```rust
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
   let f     = File::open( "dont.exist" );
   let _file = f.unwraps( &log );


   // This is equivalent. Of course you can do something else with the result after logging rather than unwrapping. This only logs
   // if the result is an error.
   //
   let g     = File::open( "dont.exist" );
   let _file = g.log( &log, Level::Critical ).unwrap();


   // Without this crate, everytime you want to unwrap, you would write something like:
   //
   let h     = File::open( "dont.exist" );

   let _file = match h
   {
      Ok ( f ) => f,
      Err( e ) => { crit!( log, "{}", e ); panic!() }
   };
}
```

