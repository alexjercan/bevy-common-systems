# Bug discovered in breach game

- STATUS: CLOSED
- PRIORITY: 100
- TAGS: bug,14_breach,crash

Description: I was on wave 3 I was shooting enemies and all of a sudden the
game crashed without any notice or anything.

```
⋊> ~/p/bevy-common-systems on master ↑ cargo run --example 14_breach
   Compiling bevy_common_systems v0.0.1 (/home/alex/personal/bevy-common-systems)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.18s
warning: the following packages contain code that will be rejected by a future version of Rust: proc-macro-error2 v2.0.1
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 1`
     Running `target/debug/examples/14_breach`
2026-07-05T12:15:52.712165Z  INFO bevy_diagnostic::system_information_diagnostics_plugin::internal: SystemInfo { os: "Linux (NixOS 26.11)", kernel: "6.18.37", cpu: "12th Gen Intel(R) Core(TM) i9-12900F", core_count: "16", memory: "31.2 GiB" }
2026-07-05T12:15:52.716377Z  WARN winit::platform_impl::linux::x11::xdisplay: error setting XSETTINGS; Xft options won't reload automatically
2026-07-05T12:15:52.780766Z  INFO bevy_render::renderer: AdapterInfo { name: "NVIDIA GeForce RTX 3060 Ti", vendor: 4318, device: 9417, device_type: DiscreteGpu, device_pci_bus_id: "0000:01:00.0", driver: "NVIDIA", driver_info: "595.84", backend: Vulkan, subgroup_min_size: 32, subgroup_max_size: 32, transient_saves_memory: false }
2026-07-05T12:15:53.289401Z  INFO bevy_pbr::cluster: GPU clustering is supported on this device.
2026-07-05T12:15:53.289521Z  INFO bevy_render::batching::gpu_preprocessing: GPU preprocessing is fully supported on this device.
2026-07-05T12:15:53.291859Z  INFO bevy_winit::system: Creating new window 14_breach (65v0)
2026-07-05T12:15:53.292235Z  INFO winit::platform_impl::linux::x11::window: Guessed window scale factor: 1
2026-07-05T12:15:53.607396Z  WARN bevy_render::view::window: Couldn't get swap chain texture after configuring. Cause: 'Outdated'
2026-07-05T12:16:36.748046Z  WARN bevy_render::view::window: Couldn't get swap chain texture after configuring. Cause: 'Outdated'
Encountered an error in command `<Enable the debug feature to see the name>`: Entity despawned: The entity with ID 622v2 is invalid; its index now has generation 3.
Note that interacting with a despawned entity is the most common cause of this error but there are others

    If you were attempting to apply a command to this entity,
    and want to handle this error gracefully, consider using `EntityCommands::queue_handled` or `queue_silenced`.
   0: from<bevy_ecs::world::error::EntityMutableFetchError>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/error/bevy_error.rs:411:28
   1: <bevy_ecs::world::error::EntityMutableFetchError as core::convert::Into<bevy_ecs::error::bevy_error::BevyError>>::into
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/core/src/convert/mod.rs:780:9
   2: <<bevy_ecs::world::error::EntityMutableFetchError as core::convert::Into<bevy_ecs::error::bevy_error::BevyError>>::into as core::ops::function::FnOnce<(bevy_ecs::world::error::EntityMutableFetchError,)>>::call_once
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/core/src/ops/function.rs:250:5
   3: <core::option::Option<bevy_ecs::world::error::EntityMutableFetchError>>::map::<bevy_ecs::error::bevy_error::BevyError, <bevy_ecs::world::error::EntityMutableFetchError as core::convert::Into<bevy_ecs::error::bevy_error::BevyError>>::into>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/core/src/option.rs:1162:29
   4: <core::result::Result<(), bevy_ecs::world::error::EntityMutableFetchError> as bevy_ecs::error::command_handling::CommandOutput>::to_err
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/error/command_handling.rs:26:20
   5: <<bevy_ecs::system::commands::entity_command::insert<bevy_common_systems::tween::TweenFinished>::{closure#0} as bevy_ecs::system::commands::entity_command::EntityCommand>::with_entity::{closure#0} as bevy_ecs::system::commands::command::Command>::handle_error::{closure#0}
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/system/commands/command.rs:90:52
   6: <<<bevy_ecs::system::commands::entity_command::insert<bevy_common_systems::tween::TweenFinished>::{closure#0} as bevy_ecs::system::commands::entity_command::EntityCommand>::with_entity::{closure#0} as bevy_ecs::system::commands::command::Command>::handle_error::{closure#0} as bevy_ecs::system::commands::command::Command>::apply
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/system/commands/command.rs:121:9
   7: {closure#0}<bevy_ecs::system::commands::command::Command::handle_error::{closure_env#0}<bevy_ecs::system::commands::entity_command::EntityCommand::with_entity::{closure_env#0}<bevy_ecs::system::commands::entity_command::insert::{closure_env#0}<bevy_common_systems::tween::TweenFinished>>>>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/world/command_queue.rs:185:33
   8: <<bevy_ecs::world::command_queue::RawCommandQueue>::push<<<bevy_ecs::system::commands::entity_command::insert<bevy_common_systems::tween::TweenFinished>::{closure#0} as bevy_ecs::system::commands::entity_command::EntityCommand>::with_entity::{closure#0} as bevy_ecs::system::commands::command::Command>::handle_error::{closure#0}>::{closure#0} as core::ops::function::FnOnce<(bevy_ptr::OwningPtr<bevy_ptr::Unaligned>, core::option::Option<core::ptr::non_null::NonNull<bevy_ecs::world::World>>, &mut usize)>>::call_once
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/core/src/ops/function.rs:250:5
   9: <bevy_ecs::world::command_queue::RawCommandQueue>::apply_or_drop_queued::{closure#0}
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/world/command_queue.rs:275:26
  10: <core::panic::unwind_safe::AssertUnwindSafe<<bevy_ecs::world::command_queue::RawCommandQueue>::apply_or_drop_queued::{closure#0}> as core::ops::function::FnOnce<()>>::call_once
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/core/src/panic/unwind_safe.rs:275:9
  11: do_call<core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::world::command_queue::{impl#5}::apply_or_drop_queued::{closure_env#0}>, ()>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/panicking.rs:575:43
  12: __rust_try
  13: catch_unwind<(), core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::world::command_queue::{impl#5}::apply_or_drop_queued::{closure_env#0}>>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/panicking.rs:543:19
  14: catch_unwind<core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::world::command_queue::{impl#5}::apply_or_drop_queued::{closure_env#0}>, ()>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/panic.rs:359:14
  15: apply_or_drop_queued
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/world/command_queue.rs:280:30
  16: <bevy_ecs::world::command_queue::CommandQueue>::apply
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/world/command_queue.rs:106:28
  17: <bevy_ecs::world::command_queue::CommandQueue as bevy_ecs::system::system_param::SystemBuffer>::apply
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/world/command_queue.rs:342:14
  18: <bevy_ecs::system::system_param::Deferred<bevy_ecs::world::command_queue::CommandQueue> as bevy_ecs::system::system_param::SystemParam>::apply
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/system/system_param.rs:1232:21
  19: <(bevy_ecs::system::system_param::Deferred<bevy_ecs::world::command_queue::CommandQueue>, &bevy_ecs::entity::EntityAllocator, &bevy_ecs::entity::Entities) as bevy_ecs::system::system_param::SystemParam>::apply
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/system/system_param.rs:2048:19
  20: <bevy_ecs::system::commands::Commands as bevy_ecs::system::system_param::SystemParam>::apply
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/system/commands/mod.rs:161:13
  21: <(bevy_ecs::change_detection::params::Res<bevy_time::time::Time>, bevy_ecs::system::commands::Commands, bevy_ecs::system::query::Query<(bevy_ecs::entity::Entity, &mut bevy_common_systems::tween::Tween<f32>)>) as bevy_ecs::system::system_param::SystemParam>::apply
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/system/system_param.rs:2048:19
  22: <bevy_ecs::system::function_system::FunctionSystem<fn(bevy_ecs::change_detection::params::Res<bevy_time::time::Time>, bevy_ecs::system::commands::Commands, bevy_ecs::system::query::Query<(bevy_ecs::entity::Entity, &mut bevy_common_systems::tween::Tween<f32>)>), (), (), bevy_common_systems::tween::advance_tween<f32>> as bevy_ecs::system::system::System>::apply_deferred
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/system/function_system.rs:715:9
  23: bevy_ecs::schedule::executor::multi_threaded::apply_deferred::{closure#0}
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/schedule/executor/multi_threaded.rs:787:20
  24: <bevy_ecs::schedule::executor::multi_threaded::apply_deferred::{closure#0} as core::ops::function::FnOnce<()>>::call_once
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/core/src/ops/function.rs:250:5
  25: <core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::schedule::executor::multi_threaded::apply_deferred::{closure#0}> as core::ops::function::FnOnce<()>>::call_once
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/core/src/panic/unwind_safe.rs:275:9
  26: do_call<core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::schedule::executor::multi_threaded::apply_deferred::{closure_env#0}>, ()>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/panicking.rs:575:43
  27: __rust_try
  28: catch_unwind<(), core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::schedule::executor::multi_threaded::apply_deferred::{closure_env#0}>>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/panicking.rs:543:19
  29: catch_unwind<core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::schedule::executor::multi_threaded::apply_deferred::{closure_env#0}>, ()>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/panic.rs:359:14
  30: bevy_ecs::schedule::executor::multi_threaded::apply_deferred
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/schedule/executor/multi_threaded.rs:786:19
  31: <bevy_ecs::schedule::executor::multi_threaded::ExecutorState>::spawn_exclusive_system_task::{closure#0}
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/schedule/executor/multi_threaded.rs:708:27
  32: <core::panic::unwind_safe::AssertUnwindSafe<<bevy_ecs::schedule::executor::multi_threaded::ExecutorState>::spawn_exclusive_system_task::{closure#0}> as core::future::future::Future>::poll
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/core/src/panic/unwind_safe.rs:300:9
  33: <futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<<bevy_ecs::schedule::executor::multi_threaded::ExecutorState>::spawn_exclusive_system_task::{closure#0}>> as core::future::future::Future>::poll::{closure#0}
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/futures-lite-2.6.1/src/future.rs:653:53
  34: <core::panic::unwind_safe::AssertUnwindSafe<<futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<<bevy_ecs::schedule::executor::multi_threaded::ExecutorState>::spawn_exclusive_system_task::{closure#0}>> as core::future::future::Future>::poll::{closure#0}> as core::ops::function::FnOnce<()>>::call_once
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/core/src/panic/unwind_safe.rs:275:9
  35: do_call<core::panic::unwind_safe::AssertUnwindSafe<futures_lite::future::{impl#11}::poll::{closure_env#0}<core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::schedule::executor::multi_threaded::{impl#5}::spawn_exclusive_system_task::{async_block_env#0}>>>, core::task::poll::Poll<()>>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/panicking.rs:575:43
  36: __rust_try
  37: catch_unwind<core::task::poll::Poll<()>, core::panic::unwind_safe::AssertUnwindSafe<futures_lite::future::{impl#11}::poll::{closure_env#0}<core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::schedule::executor::multi_threaded::{impl#5}::spawn_exclusive_system_task::{async_block_env#0}>>>>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/panicking.rs:543:19
  38: catch_unwind<core::panic::unwind_safe::AssertUnwindSafe<futures_lite::future::{impl#11}::poll::{closure_env#0}<core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::schedule::executor::multi_threaded::{impl#5}::spawn_exclusive_system_task::{async_block_env#0}>>>, core::task::poll::Poll<()>>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/panic.rs:359:14
  39: <futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<<bevy_ecs::schedule::executor::multi_threaded::ExecutorState>::spawn_exclusive_system_task::{closure#0}>> as core::future::future::Future>::poll
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/futures-lite-2.6.1/src/future.rs:653:9
  40: <async_executor::AsyncCallOnDrop<futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<<bevy_ecs::schedule::executor::multi_threaded::ExecutorState>::spawn_exclusive_system_task::{closure#0}>>, <async_executor::Executor>::spawn_inner<core::result::Result<(), alloc::boxed::Box<dyn core::any::Any + core::marker::Send>>, futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<<bevy_ecs::schedule::executor::multi_threaded::ExecutorState>::spawn_exclusive_system_task::{closure#0}>>>::{closure#0}> as core::future::future::Future>::poll
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-executor-1.13.3/src/lib.rs:1197:31
  41: <async_task::raw::RawTask<async_executor::AsyncCallOnDrop<futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<<bevy_ecs::schedule::executor::multi_threaded::ExecutorState>::spawn_exclusive_system_task::{closure#0}>>, <async_executor::Executor>::spawn_inner<core::result::Result<(), alloc::boxed::Box<dyn core::any::Any + core::marker::Send>>, futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<<bevy_ecs::schedule::executor::multi_threaded::ExecutorState>::spawn_exclusive_system_task::{closure#0}>>>::{closure#0}>, core::result::Result<(), alloc::boxed::Box<dyn core::any::Any + core::marker::Send>>, <async_executor::Executor>::schedule::{closure#0}, ()>>::run::{closure#1}
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-task-4.7.1/src/raw.rs:550:21
  42: <<async_task::raw::RawTask<async_executor::AsyncCallOnDrop<futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<<bevy_ecs::schedule::executor::multi_threaded::ExecutorState>::spawn_exclusive_system_task::{closure#0}>>, <async_executor::Executor>::spawn_inner<core::result::Result<(), alloc::boxed::Box<dyn core::any::Any + core::marker::Send>>, futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<<bevy_ecs::schedule::executor::multi_threaded::ExecutorState>::spawn_exclusive_system_task::{closure#0}>>>::{closure#0}>, core::result::Result<(), alloc::boxed::Box<dyn core::any::Any + core::marker::Send>>, <async_executor::Executor>::schedule::{closure#0}, ()>>::run::{closure#1} as core::ops::function::FnOnce<()>>::call_once
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/core/src/ops/function.rs:250:5
  43: <core::panic::unwind_safe::AssertUnwindSafe<<async_task::raw::RawTask<async_executor::AsyncCallOnDrop<futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<<bevy_ecs::schedule::executor::multi_threaded::ExecutorState>::spawn_exclusive_system_task::{closure#0}>>, <async_executor::Executor>::spawn_inner<core::result::Result<(), alloc::boxed::Box<dyn core::any::Any + core::marker::Send>>, futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<<bevy_ecs::schedule::executor::multi_threaded::ExecutorState>::spawn_exclusive_system_task::{closure#0}>>>::{closure#0}>, core::result::Result<(), alloc::boxed::Box<dyn core::any::Any + core::marker::Send>>, <async_executor::Executor>::schedule::{closure#0}, ()>>::run::{closure#1}> as core::ops::function::FnOnce<()>>::call_once
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/core/src/panic/unwind_safe.rs:275:9
  44: do_call<core::panic::unwind_safe::AssertUnwindSafe<async_task::raw::{impl#3}::run::{closure_env#1}<async_executor::AsyncCallOnDrop<futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::schedule::executor::multi_threaded::{impl#5}::spawn_exclusive_system_task::{async_block_env#0}>>, async_executor::{impl#5}::spawn_inner::{closure_env#0}<core::result::Result<(), alloc::boxed::Box<(dyn core::any::Any + core::marker::Send), alloc::alloc::Global>>, futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::schedule::executor::multi_threaded::{impl#5}::spawn_exclusive_system_task::{async_block_env#0}>>>>, core::result::Result<(), alloc::boxed::Box<(dyn core::any::Any + core::marker::Send), alloc::alloc::Global>>, async_executor::{impl#5}::schedule::{closure_env#0}, ()>>, core::task::poll::Poll<core::result::Result<(), alloc::boxed::Box<(dyn core::any::Any + core::marker::Send), alloc::alloc::Global>>>>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/panicking.rs:575:43
  45: __rust_try
  46: catch_unwind<core::task::poll::Poll<core::result::Result<(), alloc::boxed::Box<(dyn core::any::Any + core::marker::Send), alloc::alloc::Global>>>, core::panic::unwind_safe::AssertUnwindSafe<async_task::raw::{impl#3}::run::{closure_env#1}<async_executor::AsyncCallOnDrop<futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::schedule::executor::multi_threaded::{impl#5}::spawn_exclusive_system_task::{async_block_env#0}>>, async_executor::{impl#5}::spawn_inner::{closure_env#0}<core::result::Result<(), alloc::boxed::Box<(dyn core::any::Any + core::marker::Send), alloc::alloc::Global>>, futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::schedule::executor::multi_threaded::{impl#5}::spawn_exclusive_system_task::{async_block_env#0}>>>>, core::result::Result<(), alloc::boxed::Box<(dyn core::any::Any + core::marker::Send), alloc::alloc::Global>>, async_executor::{impl#5}::schedule::{closure_env#0}, ()>>>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/panicking.rs:543:19
  47: catch_unwind<core::panic::unwind_safe::AssertUnwindSafe<async_task::raw::{impl#3}::run::{closure_env#1}<async_executor::AsyncCallOnDrop<futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::schedule::executor::multi_threaded::{impl#5}::spawn_exclusive_system_task::{async_block_env#0}>>, async_executor::{impl#5}::spawn_inner::{closure_env#0}<core::result::Result<(), alloc::boxed::Box<(dyn core::any::Any + core::marker::Send), alloc::alloc::Global>>, futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::schedule::executor::multi_threaded::{impl#5}::spawn_exclusive_system_task::{async_block_env#0}>>>>, core::result::Result<(), alloc::boxed::Box<(dyn core::any::Any + core::marker::Send), alloc::alloc::Global>>, async_executor::{impl#5}::schedule::{closure_env#0}, ()>>, core::task::poll::Poll<core::result::Result<(), alloc::boxed::Box<(dyn core::any::Any + core::marker::Send), alloc::alloc::Global>>>>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/panic.rs:359:14
  48: run<async_executor::AsyncCallOnDrop<futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::schedule::executor::multi_threaded::{impl#5}::spawn_exclusive_system_task::{async_block_env#0}>>, async_executor::{impl#5}::spawn_inner::{closure_env#0}<core::result::Result<(), alloc::boxed::Box<(dyn core::any::Any + core::marker::Send), alloc::alloc::Global>>, futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<bevy_ecs::schedule::executor::multi_threaded::{impl#5}::spawn_exclusive_system_task::{async_block_env#0}>>>>, core::result::Result<(), alloc::boxed::Box<(dyn core::any::Any + core::marker::Send), alloc::alloc::Global>>, async_executor::{impl#5}::schedule::{closure_env#0}, ()>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-task-4.7.1/src/raw.rs:549:23
  49: run<()>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-task-4.7.1/src/runnable.rs:781:18
  50: {async_fn#0}
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-executor-1.13.3/src/lib.rs:739:18
  51: {async_fn#0}
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-executor-1.13.3/src/lib.rs:325:29
  52: {async_fn#0}
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_tasks-0.19.0/src/thread_executor.rs:105:39
  53: {async_block#0}<(), bevy_tasks::task_pool::{impl#2}::scope_with_executor_inner::{async_block#0}::{async_block_env#0}<bevy_ecs::schedule::executor::multi_threaded::{impl#2}::run::{closure_env#1}, ()>>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_tasks-0.19.0/src/task_pool.rs:542:45
  54: <core::panic::unwind_safe::AssertUnwindSafe<<bevy_tasks::task_pool::TaskPool>::execute_scope<(), <bevy_tasks::task_pool::TaskPool>::scope_with_executor_inner<<bevy_ecs::schedule::executor::multi_threaded::MultiThreadedExecutor as bevy_ecs::schedule::executor::SystemExecutor>::run::{closure#1}, ()>::{closure#0}::{closure#0}>::{closure#0}::{closure#0}::{closure#0}> as core::future::future::Future>::poll
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/core/src/panic/unwind_safe.rs:300:9
  55: <futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<<bevy_tasks::task_pool::TaskPool>::execute_scope<(), <bevy_tasks::task_pool::TaskPool>::scope_with_executor_inner<<bevy_ecs::schedule::executor::multi_threaded::MultiThreadedExecutor as bevy_ecs::schedule::executor::SystemExecutor>::run::{closure#1}, ()>::{closure#0}::{closure#0}>::{closure#0}::{closure#0}::{closure#0}>> as core::future::future::Future>::poll::{closure#0}
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/futures-lite-2.6.1/src/future.rs:653:53
  56: <core::panic::unwind_safe::AssertUnwindSafe<<futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<<bevy_tasks::task_pool::TaskPool>::execute_scope<(), <bevy_tasks::task_pool::TaskPool>::scope_with_executor_inner<<bevy_ecs::schedule::executor::multi_threaded::MultiThreadedExecutor as bevy_ecs::schedule::executor::SystemExecutor>::run::{closure#1}, ()>::{closure#0}::{closure#0}>::{closure#0}::{closure#0}::{closure#0}>> as core::future::future::Future>::poll::{closure#0}> as core::ops::function::FnOnce<()>>::call_once
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/core/src/panic/unwind_safe.rs:275:9
  57: do_call<core::panic::unwind_safe::AssertUnwindSafe<futures_lite::future::{impl#11}::poll::{closure_env#0}<core::panic::unwind_safe::AssertUnwindSafe<bevy_tasks::task_pool::{impl#2}::execute_scope::{async_fn#0}::{async_block#0}::{async_block_env#0}<(), bevy_tasks::task_pool::{impl#2}::scope_with_executor_inner::{async_block#0}::{async_block_env#0}<bevy_ecs::schedule::executor::multi_threaded::{impl#2}::run::{closure_env#1}, ()>>>>>, core::task::poll::Poll<!>>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/panicking.rs:575:43
  58: __rust_try
  59: catch_unwind<core::task::poll::Poll<!>, core::panic::unwind_safe::AssertUnwindSafe<futures_lite::future::{impl#11}::poll::{closure_env#0}<core::panic::unwind_safe::AssertUnwindSafe<bevy_tasks::task_pool::{impl#2}::execute_scope::{async_fn#0}::{async_block#0}::{async_block_env#0}<(), bevy_tasks::task_pool::{impl#2}::scope_with_executor_inner::{async_block#0}::{async_block_env#0}<bevy_ecs::schedule::executor::multi_threaded::{impl#2}::run::{closure_env#1}, ()>>>>>>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/panicking.rs:543:19
  60: catch_unwind<core::panic::unwind_safe::AssertUnwindSafe<futures_lite::future::{impl#11}::poll::{closure_env#0}<core::panic::unwind_safe::AssertUnwindSafe<bevy_tasks::task_pool::{impl#2}::execute_scope::{async_fn#0}::{async_block#0}::{async_block_env#0}<(), bevy_tasks::task_pool::{impl#2}::scope_with_executor_inner::{async_block#0}::{async_block_env#0}<bevy_ecs::schedule::executor::multi_threaded::{impl#2}::run::{closure_env#1}, ()>>>>>, core::task::poll::Poll<!>>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/panic.rs:359:14
  61: <futures_lite::future::CatchUnwind<core::panic::unwind_safe::AssertUnwindSafe<<bevy_tasks::task_pool::TaskPool>::execute_scope<(), <bevy_tasks::task_pool::TaskPool>::scope_with_executor_inner<<bevy_ecs::schedule::executor::multi_threaded::MultiThreadedExecutor as bevy_ecs::schedule::executor::SystemExecutor>::run::{closure#1}, ()>::{closure#0}::{closure#0}>::{closure#0}::{closure#0}::{closure#0}>> as core::future::future::Future>::poll
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/futures-lite-2.6.1/src/future.rs:653:9
  62: {async_block#0}<(), bevy_tasks::task_pool::{impl#2}::scope_with_executor_inner::{async_block#0}::{async_block_env#0}<bevy_ecs::schedule::executor::multi_threaded::{impl#2}::run::{closure_env#1}, ()>>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_tasks-0.19.0/src/task_pool.rs:545:77
  63: <futures_lite::future::Or<<bevy_tasks::task_pool::TaskPool>::scope_with_executor_inner<<bevy_ecs::schedule::executor::multi_threaded::MultiThreadedExecutor as bevy_ecs::schedule::executor::SystemExecutor>::run::{closure#1}, ()>::{closure#0}::{closure#0}, <bevy_tasks::task_pool::TaskPool>::execute_scope<(), <bevy_tasks::task_pool::TaskPool>::scope_with_executor_inner<<bevy_ecs::schedule::executor::multi_threaded::MultiThreadedExecutor as bevy_ecs::schedule::executor::SystemExecutor>::run::{closure#1}, ()>::{closure#0}::{closure#0}>::{closure#0}::{closure#0}> as core::future::future::Future>::poll
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/futures-lite-2.6.1/src/future.rs:454:46
  64: {async_fn#0}<(), bevy_tasks::task_pool::{impl#2}::scope_with_executor_inner::{async_block#0}::{async_block_env#0}<bevy_ecs::schedule::executor::multi_threaded::{impl#2}::run::{closure_env#1}, ()>>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_tasks-0.19.0/src/task_pool.rs:548:41
  65: {async_block#0}<bevy_ecs::schedule::executor::multi_threaded::{impl#2}::run::{closure_env#1}, ()>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_tasks-0.19.0/src/task_pool.rs:459:85
  66: {closure#0}<alloc::vec::Vec<(), alloc::alloc::Global>, bevy_tasks::task_pool::{impl#2}::scope_with_executor_inner::{async_block_env#0}<bevy_ecs::schedule::executor::multi_threaded::{impl#2}::run::{closure_env#1}, ()>>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/futures-lite-2.6.1/src/future.rs:96:35
  67: try_with<core::cell::RefCell<(parking::Parker, core::task::wake::Waker)>, futures_lite::future::block_on::{closure_env#0}<alloc::vec::Vec<(), alloc::alloc::Global>, bevy_tasks::task_pool::{impl#2}::scope_with_executor_inner::{async_block_env#0}<bevy_ecs::schedule::executor::multi_threaded::{impl#2}::run::{closure_env#1}, ()>>, alloc::vec::Vec<(), alloc::alloc::Global>>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/thread/local.rs:463:12
  68: <std::thread::local::LocalKey<core::cell::RefCell<(parking::Parker, core::task::wake::Waker)>>>::with::<futures_lite::future::block_on<alloc::vec::Vec<()>, <bevy_tasks::task_pool::TaskPool>::scope_with_executor_inner<<bevy_ecs::schedule::executor::multi_threaded::MultiThreadedExecutor as bevy_ecs::schedule::executor::SystemExecutor>::run::{closure#1}, ()>::{closure#0}>::{closure#0}, alloc::vec::Vec<()>>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/thread/local.rs:427:20
  69: block_on<alloc::vec::Vec<(), alloc::alloc::Global>, bevy_tasks::task_pool::{impl#2}::scope_with_executor_inner::{async_block_env#0}<bevy_ecs::schedule::executor::multi_threaded::{impl#2}::run::{closure_env#1}, ()>>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/futures-lite-2.6.1/src/future.rs:75:11
  70: <bevy_tasks::task_pool::TaskPool>::scope_with_executor_inner::<<bevy_ecs::schedule::executor::multi_threaded::MultiThreadedExecutor as bevy_ecs::schedule::executor::SystemExecutor>::run::{closure#1}, ()>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_tasks-0.19.0/src/task_pool.rs:413:13
  71: <bevy_tasks::task_pool::TaskPool>::scope_with_executor::<<bevy_ecs::schedule::executor::multi_threaded::MultiThreadedExecutor as bevy_ecs::schedule::executor::SystemExecutor>::run::{closure#1}, ()>::{closure#0}
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_tasks-0.19.0/src/task_pool.rs:343:22
  72: try_with<alloc::sync::Arc<bevy_tasks::thread_executor::ThreadExecutor, alloc::alloc::Global>, bevy_tasks::task_pool::{impl#2}::scope_with_executor::{closure_env#0}<bevy_ecs::schedule::executor::multi_threaded::{impl#2}::run::{closure_env#1}, ()>, alloc::vec::Vec<(), alloc::alloc::Global>>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/thread/local.rs:463:12
  73: <std::thread::local::LocalKey<alloc::sync::Arc<bevy_tasks::thread_executor::ThreadExecutor>>>::with::<<bevy_tasks::task_pool::TaskPool>::scope_with_executor<<bevy_ecs::schedule::executor::multi_threaded::MultiThreadedExecutor as bevy_ecs::schedule::executor::SystemExecutor>::run::{closure#1}, ()>::{closure#0}, alloc::vec::Vec<()>>
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/std/src/thread/local.rs:427:20
  74: <bevy_tasks::task_pool::TaskPool>::scope_with_executor::<<bevy_ecs::schedule::executor::multi_threaded::MultiThreadedExecutor as bevy_ecs::schedule::executor::SystemExecutor>::run::{closure#1}, ()>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_tasks-0.19.0/src/task_pool.rs:339:31
  75: run
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/schedule/executor/multi_threaded.rs:274:57
  76: <bevy_ecs::schedule::schedule::Schedule>::run
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/schedule/schedule.rs:577:14
  77: <bevy_ecs::world::World>::try_run_schedule::<bevy_ecs::intern::Interned<dyn bevy_ecs::schedule::set::ScheduleLabel>>::{closure#0}
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/world/mod.rs:3908:61
  78: try_schedule_scope<(), bevy_ecs::intern::Interned<dyn bevy_ecs::schedule::set::ScheduleLabel>, bevy_ecs::world::{impl#4}::try_run_schedule::{closure_env#0}<bevy_ecs::intern::Interned<dyn bevy_ecs::schedule::set::ScheduleLabel>>>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/world/mod.rs:3841:21
  79: <bevy_ecs::world::World>::try_run_schedule::<bevy_ecs::intern::Interned<dyn bevy_ecs::schedule::set::ScheduleLabel>>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/world/mod.rs:3908:14
  80: <bevy_app::main_schedule::Main>::run_main::{closure#1}
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_app-0.19.0/src/main_schedule.rs:302:31
  81: try_resource_scope<bevy_app::main_schedule::MainScheduleOrder, (), bevy_app::main_schedule::{impl#2}::run_main::{closure_env#1}>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/world/mod.rs:3006:22
  82: <bevy_ecs::world::World>::resource_scope::<bevy_app::main_schedule::MainScheduleOrder, (), <bevy_app::main_schedule::Main>::run_main::{closure#1}>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/world/mod.rs:2852:14
  83: <bevy_app::main_schedule::Main>::run_main
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_app-0.19.0/src/main_schedule.rs:300:15
  84: <<bevy_app::main_schedule::Main>::run_main as core::ops::function::FnMut<(&mut bevy_ecs::world::World, bevy_ecs::system::system_param::Local<bool>)>>::call_mut
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/core/src/ops/function.rs:166:5
  85: <&mut <bevy_app::main_schedule::Main>::run_main as core::ops::function::FnMut<(&mut bevy_ecs::world::World, bevy_ecs::system::system_param::Local<bool>)>>::call_mut
             at /nix/store/jxrz94p6akmn9jpw0gjmydf1r1cjjzlq-rust-default-1.98.0-nightly-2026-06-19/lib/rustlib/src/rust/library/core/src/ops/function.rs:298:21
  86: <_ as bevy_ecs::system::exclusive_function_system::ExclusiveSystemParamFunction<fn(_) -> _>>::run::call_inner::<(), bevy_ecs::system::system_param::Local<bool>, &mut <bevy_app::main_schedule::Main>::run_main>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/system/exclusive_function_system.rs:261:21
  87: <<bevy_app::main_schedule::Main>::run_main as bevy_ecs::system::exclusive_function_system::ExclusiveSystemParamFunction<fn(bevy_ecs::system::system_param::Local<bool>)>>::run
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/system/exclusive_function_system.rs:264:17
  88: <bevy_ecs::system::exclusive_function_system::ExclusiveFunctionSystem<fn(bevy_ecs::system::system_param::Local<bool>), (), <bevy_app::main_schedule::Main>::run_main> as bevy_ecs::system::system::System>::run_unsafe::{closure#0}
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/system/exclusive_function_system.rs:135:33
  89: <bevy_ecs::world::World>::last_change_tick_scope::<core::result::Result<(), bevy_ecs::system::system::RunSystemError>, <bevy_ecs::system::exclusive_function_system::ExclusiveFunctionSystem<fn(bevy_ecs::system::system_param::Local<bool>), (), <bevy_app::main_schedule::Main>::run_main> as bevy_ecs::system::system::System>::run_unsafe::{closure#0}>
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/world/mod.rs:3315:9
  90: <bevy_ecs::system::exclusive_function_system::ExclusiveFunctionSystem<fn(bevy_ecs::system::system_param::Local<bool>), (), <bevy_app::main_schedule::Main>::run_main> as bevy_ecs::system::system::System>::run_unsafe
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/system/exclusive_function_system.rs:113:15
  91: <bevy_ecs::system::exclusive_function_system::ExclusiveFunctionSystem<fn(bevy_ecs::system::system_param::Local<bool>), (), <bevy_app::main_schedule::Main>::run_main> as bevy_ecs::system::system::System>::run_without_applying_deferred
             at /home/alex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.19.0/src/system/system.rs:147:23
note: Some "noisy" backtrace lines have been filtered out. Run with `BEVY_BACKTRACE=full` for a verbose backtrace.

Encountered a panic when applying buffers for system `<Enable the debug feature to see the name>`!
Encountered a panic in system `<Enable the debug feature to see the name>`!
Encountered a panic in system `<Enable the debug feature to see the name>`!
⏎
⋊> ~/p/bevy-common-systems on master ↑
```

## Resolution

Fixed in `tasks/20260705-155230` (branch fix/tween-despawn-race). Root cause: the crate
`tween` module's `advance_tween` used `commands.entity(entity).insert(TweenFinished)` /
`.remove().insert()` / `.despawn()` on completion, which panics if the entity was
despawned before that command buffer flushed (a `feedback/flash` tween on a killed enemy,
a `ui/popup` fade whose node despawns). Switched to the fallible `try_insert` /
`try_remove` / `try_despawn` (no-ops on a stale entity). Covered by a deterministic
regression test that reproduces the exact "Entity despawned" panic (auto-inserted sync
points disabled so the despawn and the tween completion land in the same flush) and a
25s sustained-combat breach autopilot run with no crash.
