use leptos::either::either;
use leptos::prelude::*;
use crate::model::hanzi::Hanzi;
use crate::model::hanzi_strokes::HanziStrokes;

/// Converts a pinyin string with diacritics to one with a number. e.g., "rén" -> "ren2"
fn pinyin_diacritic_to_numbered(pinyin: &str) -> String {
    let mut base = String::new();
    let mut tone = '5'; // Default to neutral tone

    // Maps each diacritic to its base character and tone number
    for c in pinyin.chars() {
        let (base_char, found_tone) = match c {
            'ā' => ('a', '1'), 'á' => ('a', '2'), 'ǎ' => ('a', '3'), 'à' => ('a', '4'),
            'ō' => ('o', '1'), 'ó' => ('o', '2'), 'ǒ' => ('o', '3'), 'ò' => ('o', '4'),
            'ē' => ('e', '1'), 'é' => ('e', '2'), 'ě' => ('e', '3'), 'è' => ('e', '4'),
            'ī' => ('i', '1'), 'í' => ('i', '2'), 'ǐ' => ('i', '3'), 'ì' => ('i', '4'),
            'ū' => ('u', '1'), 'ú' => ('u', '2'), 'ǔ' => ('u', '3'), 'ù' => ('u', '4'),
            'ǖ' => ('v', '1'), 'ǘ' => ('v', '2'), 'ǚ' => ('v', '3'), 'ǜ' => ('v', '4'), 'ü' => ('v', '5'),
            'ḿ' => ('m', '2'),
            // Not a toned character
            _ => (c, '0'),
        };
        
        base.push(base_char);
        if found_tone != '0' {
            tone = found_tone;
        }
    }
    
    if tone != '5' {
        base.push(tone);
    }
    base
}

/// A component for the first exercise type: guessing pronunciation and meaning.
#[component]
pub fn PronunciationMeaningExercise(
    #[prop(into)] character: Signal<char>,
    #[prop(into)] on_complete: Callback<()>,
) -> impl IntoView {
    let exercise_data = LocalResource::new(move || async move {
        let hanzi_data = Hanzi::get_one_from_db(character.get()).await.ok().flatten();
        let stroke_data = HanziStrokes::get_one_from_db(character.get()).await.ok().flatten();
        (hanzi_data, stroke_data)
    });

    view! {
        <Suspense fallback=move || view!{ <div class="skeleton w-full h-72"></div> }>
            {Suspend::new(async move {
                let data = exercise_data.await;
                either!(data,
                    (Some(hanzi), Some(strokes)) => {
                        let pinyin_inputs = (0..hanzi.pinyin.len())
                            .map(|i| (i, signal(String::new())))
                            .collect::<Vec<_>>();
                        let (meaning_input, set_meaning_input) = signal(String::new());
                        let (is_checked, set_is_checked) = signal(false);
                        let (pinyin_correct, set_pinyin_correct) = signal(Option::<bool>::None);
                        let (meaning_correct, set_meaning_correct) = signal(Option::<bool>::None);
                        
                        let all_answers_correct = Signal::derive(move || {
                            pinyin_correct.get() == Some(true) && meaning_correct.get() == Some(true)
                        });

                        let maybe_hint_text = hanzi.etymology.as_ref().and_then(|e| {
                            if e.hint.is_empty() { None } else { Some(e.hint.clone()) }
                        });

                        // Create clones for each closure that needs ownership of the data.
                        let hanzi_for_pinyin_display = hanzi.clone();
                        let hanzi_for_definition_display = hanzi.clone();
                        let hanzi_for_check = hanzi.clone();
                        let pinyin_inputs_for_check = pinyin_inputs.clone();
                        
                        view! {
                            <div>
                                <h2 class="card-title mb-4">"What are the pronunciation and meaning of this character?"</h2>
                                <div class="flex flex-col md:flex-row gap-8 items-center justify-center">
                                    <div class="w-48 h-48 bg-base-200 rounded-lg p-2">
                                        <svg viewBox="0 0 1024 1024" xmlns="http://www.w3.org/2000/svg">
                                            <g transform="scale(1, -1) translate(0, -900)">
                                                {strokes.strokes.into_iter().map(|d| {
                                                    view! { <path fill="currentColor" d=d></path> }
                                                }).collect::<Vec<_>>()}
                                            </g>
                                        </svg>
                                    </div>
                                    <div class="form-control gap-4 w-full md:w-auto">
                                        <div>
                                            <label class="label"><span class="label-text">"Pronunciation (Pinyin)"</span></label>
                                            {pinyin_inputs.into_iter().map(|(_, (input, set_input))| {
                                                view! {
                                                    <input
                                                        type="text"
                                                        placeholder="e.g., ren2"
                                                        class="input input-bordered w-full mb-2"
                                                        class=("input-success", move || pinyin_correct.get() == Some(true))
                                                        class=("input-error", move || pinyin_correct.get() == Some(false))
                                                        prop:value=input
                                                        on:input=move |ev| set_input.set(event_target_value(&ev))
                                                    />
                                                }
                                            }).collect::<Vec<_>>()}
                                            {move || either!(is_checked.get(),
                                                true => view! {
                                                    <div class="text-sm mt-1 opacity-80">
                                                        "Correct: " <span class="font-semibold">{
                                                            hanzi_for_pinyin_display.pinyin.iter()
                                                                .map(|p| format!("{} ({})", p, pinyin_diacritic_to_numbered(p)))
                                                                .collect::<Vec<_>>().join(", ")
                                                        }</span>
                                                    </div>
                                                },
                                                false => view! { <div class="h-6"></div> }
                                            )}
                                        </div>
                                        <div>
                                            <label class="label"><span class="label-text">"Meaning"</span></label>
                                            <input
                                                type="text"
                                                placeholder="e.g., person, man"
                                                class="input input-bordered w-full"
                                                class=("input-success", move || meaning_correct.get() == Some(true))
                                                class=("input-error", move || meaning_correct.get() == Some(false))
                                                prop:value=meaning_input
                                                on:input=move |ev| set_meaning_input(event_target_value(&ev))
                                            />
                                            {move || either!(is_checked.get(),
                                                true => view! {
                                                    <div class="text-sm mt-1 opacity-80">
                                                        "Correct: " <span class="font-semibold">{hanzi_for_definition_display.definition.as_deref().unwrap_or("")}</span>
                                                    </div>
                                                },
                                                false => view! { <div class="h-6"></div> }
                                            )}
                                        </div>
                                    </div>
                                </div>
                                <div class="mt-4 text-center">
                                    {move || either!(maybe_hint_text.clone(),
                                        Some(hint_text) => {
                                            let (hint_visible, set_hint_visible) = signal(false);
                                            view! {
                                                {move || either!(hint_visible.get(),
                                                    false => view! {
                                                        <button
                                                            class="btn btn-xs btn-ghost"
                                                            on:click=move |_| set_hint_visible.set(true)
                                                        >
                                                            "Show Hint"
                                                        </button>
                                                    },
                                                    true => view! {
                                                        <p class="opacity-80">
                                                            <span class="font-semibold">"Hint: "</span>
                                                            {hint_text.clone()}
                                                        </p>
                                                    }
                                                )}
                                            }
                                        },
                                        None => view! { <div/> } // Render empty div if no hint
                                    )}
                                </div>
                                <div class="card-actions justify-end mt-6">
                                    {move || either!(is_checked.get(),
                                        true => {
                                            let is_correct = all_answers_correct.get();
                                            view! {
                                                <button
                                                    class="btn"
                                                    class=("btn-success", is_correct)
                                                    class=("btn-warning", !is_correct)
                                                    on:click=move |_| on_complete.run(())
                                                >
                                                    "Continue"
                                                </button>
                                            }
                                        },
                                        false => {
                                            let hanzi_for_click = hanzi_for_check.clone();
                                            let pinyin_inputs_click = pinyin_inputs_for_check.clone();
                                            view! {
                                                <button
                                                    class="btn btn-secondary"
                                                    on:click=move |_| {
                                                        
                                                        let mut user_inputs: Vec<_> = pinyin_inputs_click.iter()
                                                            .map(|(_, (input, _))| input.get_untracked().to_lowercase())
                                                            .collect();
                                                        user_inputs.sort();

                                                        let mut correct_diacritic: Vec<_> = hanzi_for_click.pinyin.iter().map(|p| p.to_lowercase()).collect();
                                                        correct_diacritic.sort();

                                                        let mut correct_numbered: Vec<_> = hanzi_for_click.pinyin.iter().map(|p| pinyin_diacritic_to_numbered(p)).collect();
                                                        correct_numbered.sort();

                                                        set_pinyin_correct(Some(user_inputs == correct_diacritic || user_inputs == correct_numbered));
                                                        let correct_meaning = hanzi_for_click.definition.as_deref().unwrap_or_default().to_lowercase();
                                                        let m_correct = !correct_meaning.is_empty() && correct_meaning.contains(&meaning_input.get_untracked().to_lowercase());
                                                        set_meaning_correct(Some(m_correct));
                                                        
                                                        set_is_checked(true);
                                                    }
                                                >
                                                    "Check Answer"
                                                </button>
                                            }
                                        }
                                    )}
                                </div>
                            </div>
                        }
                    },
                    _ => view! { <p>"Could not load data for this character."</p> }
                )
            })}
        </Suspense>
    }
}