use leptos::either::either;
use leptos::prelude::*;
use leptos_meta::*;
use crate::components::exercises::handwriting::HandwritingExercise;
use crate::components::exercises::pronunciation_meaning::PronunciationMeaningExercise;
use crate::components::exercises::types::Exercise;
use crate::components::nav::Nav;


/// The main component for the learning page, managing the sequence of exercises.
#[component]
pub fn Learning() -> impl IntoView {
    // A static list of exercises for the learning session.
    // We'll start with our new exercise type and a placeholder for the next one.
    let exercises = vec![
        Exercise::Handwriting('猫'),
        Exercise::PronunciationAndMeaning('中'),
        Exercise::PronunciationAndMeaning('人'),
        Exercise::PronunciationAndMeaning('了'),
        Exercise::PronunciationAndMeaning('猫'),
        Exercise::Placeholder,
    ];
    let exercises_len = exercises.len();

    // State to track the current exercise index.
    let (current_exercise_index, set_current_exercise_index) = signal(0);

    // Calculate progress percentage.
    let progress = move || (current_exercise_index.get() + 1) as f32 / exercises_len as f32 * 100.0;

    let advance_exercise = move || {
        set_current_exercise_index.update(|i| {
            if *i < exercises_len -1 {
                *i += 1
            }
        });
    };

    view! {
        <main>
            <Title text="中 Char - Learning"/>
            <Nav />
            <div class="p-6 max-w-4xl mx-auto">
                <h1 class="text-2xl font-bold mb-4">"Learning Exercises"</h1>

                <progress class="progress progress-primary w-full mb-6" value=move || progress().to_string() max="100"></progress>

                <div class="card bg-base-100">
                    <div class="card-body min-h-[24rem]">
                        {move || {
                            let current_exercise = exercises.get(current_exercise_index.get());
                            either!(current_exercise,
                                Some(exercise) => either!(exercise,
                                    Exercise::PronunciationAndMeaning(char_to_learn) => view! {
                                        <PronunciationMeaningExercise character=*char_to_learn on_complete=advance_exercise />
                                    },
                                    Exercise::Handwriting(char_to_learn) => view! { // <-- Add this match arm
                                        <HandwritingExercise character=*char_to_learn on_complete=advance_exercise />
                                    },
                                    Exercise::Placeholder => view! {
                                        <div class="flex flex-col items-center justify-center h-full">
                                            <h2 class="card-title">"Next Exercise Type"</h2>
                                            <p>"This is a placeholder for the next type of exercise."</p>
                                        </div>
                                    }
                                ),
                                None => view! {
                                    <div class="flex flex-col items-center justify-center h-full">
                                         <h2 class="card-title">"You've completed all exercises!"</h2>
                                         <p>"Refresh the page to start again."</p>
                                    </div>
                                }
                            )
                        }}
                    </div>
                </div>
            </div>
        </main>
    }
}