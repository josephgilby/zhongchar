use leptos::prelude::*;
use leptos::ev::Event;
use leptos_meta::*;
use leptos::html::Dialog;
use crate::components::exercise_builders::type_1a::ExerciseBuilder1a;
use crate::components::nav::Nav;

use crate::model::{error::ArcZhongCharError, exercise::Exercise};

#[component]
pub fn Exercises() -> impl IntoView {
    let add_exercise_action = Action::new_unsync_local(move |new_exercise: &Exercise| {
        let ex = new_exercise.clone();
        async move {
            Exercise::add_to_db(&ex).await
        }
    });

    let exercises_fetched = LocalResource::new(move || {
        add_exercise_action.version().get(); // <-- Tracks the action
        async move {
            Exercise::get_all_from_db()
                .await
                .map_err(|e| ArcZhongCharError::from(e))
        }
    });

    // 1. Create a resource to fetch all exercises from the database.
    // let exercises_fetched = LocalResource::new(move || async move {
    //     Exercise::get_all_from_db()
    //         .await
    //         .map_err(|e| ArcZhongCharError::from(e))
    // });

    let dialog_ref: NodeRef<Dialog> = NodeRef::new();

    let open_modal = move |_| {
        if let Some(dialog) = dialog_ref.get() {
            let _ = dialog.show_modal();
        }
    };

    let close_modal = move || {
        if let Some(dialog) = dialog_ref.get() {
            let _ = dialog.close();
        }
    };

    let on_save_handler = move |new_exercise: Exercise| {
        add_exercise_action.dispatch(new_exercise); // Dispatch the action to save
        close_modal(); // Close the modal
    };

    let on_cancel_handler = move || {
        close_modal();
    };

    view! {
        <main>
            <Title text="中 Char - Exercises"/>
            <Nav/>
            <div class="w-full h-dvh absolute top-0 flex flex-col">
                <div class="w-full h-16 flex-none"></div>
                <div id="radical-table-container" class="w-full flex-grow overflow-x-auto overflow-y-auto">
                <table class="table table-zebra table-pin-rows w-full">
                    <thead>
                        <tr>
                            <th>"Exercise ID"</th>
                            <th>"Type"</th>
                            <th>"Concept ID"</th>
                            <th>"Prompt"</th>
                            <th>"Pinyin"</th>
                            <th>"Meaning"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <Suspense fallback=move || {
                            let vec_20: Vec<i32> = (0..20).collect();
                            vec_20.into_iter().map(|n| {
                                if n % 2 == 0 {
                                    view!{<tr><td colspan="6">"\u{00A0}"</td></tr>}
                                } else {
                                    view!{<tr><td colspan="6" class="skeleton">"\u{00A0}"</td></tr>}
                                }
                            })
                            .collect::<Vec<_>>()
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
                                    exercises_fetched.await.map(|exercises| { // Changed this line
                                        view! {
                                            <> // Added Fragment here
                                                {
                                                    exercises.into_iter().map(|exercise| {
                                                        match exercise {
                                                            Exercise::Type1a(ex) => view! {
                                                                <tr>
                                                                    <td class="font-mono text-xs">{ex.exercise_id}</td>
                                                                    <td>"1a: Recognition"</td>
                                                                    <td class="font-mono text-xs">{ex.target_concept_id}</td>
                                                                    <td class="font-mono text-xl">{ex.prompt}</td>
                                                                    <td>{ex.pinyin.join(", ")}</td>
                                                                    <td>{ex.meaning}</td>
                                                                </tr>
                                                            },
                                                            // Add other arms here as you create more types
                                                            // _ => view! { <tr><td colspan="6">"Unknown exercise type"</td></tr> }
                                                        }
                                                    }).collect::<Vec<_>>()
                                                }
                                            </> // Close Fragment here
                                        }
                                    })
                                })}

                            </ErrorBoundary>
                        </Suspense>
                    </tbody>
                </table>
                </div>

            </div>
            <div class="fab">
                <button 
                    class="btn btn-circle btn-lg btn-primary"
                    on:click=open_modal
                >
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
                    </svg>
                </button>
                // <div tabindex="0" role="button" class="btn btn-circle btn-lg btn-primary">
                //     <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                //         <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
                //     </svg>
                // </div>
            </div>

            <dialog 
                node_ref=dialog_ref 
                class="modal"
                on:cancel=move |_: Event| on_cancel_handler()
            >
                <div class="modal-box">
                    <ExerciseBuilder1a 
                        on_save=on_save_handler 
                        on_cancel=on_cancel_handler 
                    />
                </div>
            </dialog>

            // <dialog node_ref=dialog_ref class="modal">
            //     <div class="modal-box dark:bg-neutral">
            //         <h3 class="font-bold text-lg dark:text-neutral-content">"Create New Exercise"</h3>
            //         <p class="py-4 dark:text-neutral-content">"Your form will go here."</p>
            //         <div class="modal-action">
            //             // We must provide a way to close the dialog
            //             <button class="btn btn-primary" on:click=close_modal>
            //                 "Close"
            //             </button>
            //         </div>
            //     </div>
            // </dialog>
        </main>
    }
}