use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};
use crate::model::error::ArcZhongCharError;
use crate::model::hanzi::Hanzi;
use crate::model::hanzi_strokes::HanziStrokes;
use crate::model::radical::Radical;
use crate::pages::home::Home;
use crate::pages::radicals::Radicals;
use crate::pages::decomposition::Decomposition;
use crate::pages::exercises::Exercises;
use crate::pages::learning::Learning;

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppState {
    Initializing,
    Ready,
}


#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let base_url = option_env!("BASE_URL").unwrap_or("");

    let seeding_radicals = LocalResource::new( move || async move {
        Radical::seed_if_needed()
            .await
            .map_err(|e| ArcZhongCharError::from(e) )
    });
    let seeding_hanzis = LocalResource::new( move || async move {
        Hanzi::seed_if_needed()
            .await
            .map_err(|e| ArcZhongCharError::from(e) )
    });

    let seeding_hanzi_strokes = LocalResource::new( move || async move {
        HanziStrokes::seed_if_needed()
            .await
            .map_err(|e| ArcZhongCharError::from(e) )
    });

    view! {
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Suspense fallback=move || {
            view!{}
        }>
            <ErrorBoundary fallback=|errors| view! {
                <div class="error">
                    <p>"Not a number! Errors: "</p>
                    <ul>
                        {move || errors.get()
                            .into_iter()
                            .map(|(_, e)| view! { <li>{e.to_string()}</li>})
                            .collect::<Vec<_>>()
                        }
                    </ul>
                </div>
            }>
                { move || Suspend::new ( async move {
                    let _ = seeding_radicals.await?;
                    let _ = seeding_hanzis.await?;
                    let _ = seeding_hanzi_strokes.await?;
                    Ok::<_, ArcZhongCharError>(view! {
                        <Router base=base_url>
                            <Routes fallback=|| "Page not found.">
                                <Route path=path!("/") view=Home/>
                                <Route path=path!("/radicals") view=Radicals/>
                                <Route path=path!("/decomposition") view=Decomposition/>
                                <Route path=path!("/exercises") view=Exercises/>
                                <Route path=path!("/learning") view=Learning/>
                            </Routes>
                        </Router>
                    })

                })}
            </ErrorBoundary>
        </Suspense>

    }
}


