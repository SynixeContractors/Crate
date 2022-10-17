#![feature(prelude_import)]
#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use synixe_events::recruiting::executions::{Request, Response};
use synixe_proc::events_request;
use tokio_simple_scheduler::{Job, Scheduler};
#[macro_use]
extern crate log;
fn main() {
    let body = async {
        opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
        bootstrap::logger::init();
        let mut sched = Scheduler::default();
        bootstrap::NC::get().await;
        sched . add (Job :: new ("Recruiting - check steam forums for new posts" , "0 */10 * * * *" , | | { Box :: pin (async { if let Err (e) = async { use synixe_events :: Evokable ; let body = Request :: CheckSteam { } ; let path = body . self_path () ; { let lvl = :: log :: Level :: Debug ; if lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () { :: log :: __private_api_log (:: core :: fmt :: Arguments :: new_v1 (& ["requesting on "] , & [:: core :: fmt :: ArgumentV1 :: new_debug (& path)]) , lvl , & ("synixe_scheduler" , "synixe_scheduler" , "synixe-scheduler/src/main.rs" , 30u32) , :: log :: __private_api :: Option :: None) ; } } ; let mut trace_body = synixe_events :: Wrapper :: new (body) ; synixe_events :: opentelemetry :: global :: get_text_map_propagator (| injector | { injector . inject_context (& synixe_events :: opentelemetry :: Context :: current () , & mut trace_body) ; }) ; let response = bootstrap :: NC :: get () . await . request_timeout (path , synixe_events :: serde_json :: to_vec (& trace_body) . unwrap () , std :: time :: Duration :: from_secs (2)) . await ; match response { Ok (response) => { Ok ({ let wrapper = :: synixe_events :: serde_json :: from_slice :: < :: synixe_events :: Wrapper < Response :: CheckSteam > > (& response . data) . unwrap () ; let parent_context = :: synixe_events :: opentelemetry :: global :: get_text_map_propagator (| propagator | { propagator . extract (& wrapper) }) ; (wrapper . into_parts () , parent_context) }) } Err (e) => { { let lvl = :: log :: Level :: Error ; if lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () { :: log :: __private_api_log (:: core :: fmt :: Arguments :: new_v1 (& ["Error in request " , ": "] , & [:: core :: fmt :: ArgumentV1 :: new_display (& path) , :: core :: fmt :: ArgumentV1 :: new_display (& e)]) , lvl , & ("synixe_scheduler" , "synixe_scheduler" , "synixe-scheduler/src/main.rs" , 30u32) , :: log :: __private_api :: Option :: None) ; } } ; Err (e . to_string ()) } } } . await { { let lvl = :: log :: Level :: Error ; if lvl <= :: log :: STATIC_MAX_LEVEL && lvl <= :: log :: max_level () { :: log :: __private_api_log (:: core :: fmt :: Arguments :: new_v1 (& ["error checking on steam: "] , & [:: core :: fmt :: ArgumentV1 :: new_debug (& e)]) , lvl , & ("synixe_scheduler" , "synixe_scheduler" , "synixe-scheduler/src/main.rs" , 32u32) , :: log :: __private_api :: Option :: None) ; } } ; } }) }) . unwrap ()) ;
        sched.add(
            Job::new(
                "Recruiting - check reddit findaunit for new posts",
                "0 */10 * * * *",
                || {
                    Box::pin(async {
                        if let Err(e) = ().await {
                            {
                                let lvl = ::log::Level::Error;
                                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                    ::log::__private_api_log(
                                        ::core::fmt::Arguments::new_v1(
                                            &["error checking on reddit: "],
                                            &[::core::fmt::ArgumentV1::new_debug(&e)],
                                        ),
                                        lvl,
                                        &(
                                            "synixe_scheduler",
                                            "synixe_scheduler",
                                            "synixe-scheduler/src/main.rs",
                                            46u32,
                                        ),
                                        ::log::__private_api::Option::None,
                                    );
                                }
                            };
                        }
                    })
                },
            )
            .unwrap(),
        );
        sched.add(
            Job::new(
                "Recruiting - reddit findaunit post",
                "0 0 23 * * Thu,Fri,Sat",
                || {
                    Box::pin(async {
                        if let Err(e) = ().await {
                            {
                                let lvl = ::log::Level::Error;
                                if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                                    ::log::__private_api_log(
                                        ::core::fmt::Arguments::new_v1(
                                            &["error posting to reddit: "],
                                            &[::core::fmt::ArgumentV1::new_debug(&e)],
                                        ),
                                        lvl,
                                        &(
                                            "synixe_scheduler",
                                            "synixe_scheduler",
                                            "synixe-scheduler/src/main.rs",
                                            60u32,
                                        ),
                                        ::log::__private_api::Option::None,
                                    );
                                }
                            };
                        }
                    })
                },
            )
            .unwrap(),
        );
        sched.start().await;
        {
            let lvl = ::log::Level::Info;
            if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                ::log::__private_api_log(
                    ::core::fmt::Arguments::new_v1(&["Done!"], &[]),
                    lvl,
                    &(
                        "synixe_scheduler",
                        "synixe_scheduler",
                        "synixe-scheduler/src/main.rs",
                        70u32,
                    ),
                    ::log::__private_api::Option::None,
                );
            }
        };
    };
    #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
    {
        return tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
