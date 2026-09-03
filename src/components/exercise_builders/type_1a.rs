use leptos::ev::SubmitEvent;
//use leptos::event_target_value;
use leptos::prelude::*;
use crate::model::exercise::{Exercise, Exercise1a};

#[component]
pub fn ExerciseBuilder1a(
    #[prop(into)] on_save: Callback<(Exercise,)>,
    #[prop(into)] on_cancel: Callback<()>
) -> impl IntoView {

    // --- INTERNAL FORM STATE ---
    let (prompt, set_prompt) = signal(String::new());
    let (pinyin, set_pinyin) = signal(String::new());
    let (meaning, set_meaning) = signal(String::new());

    // --- EVENT HANDLERS ---
    let on_submit_handler = move |ev: SubmitEvent| {
        ev.prevent_default(); 
        
        let p = prompt.get_untracked();
        if p.is_empty() { return; } 

        let pinyin_vec = pinyin.get_untracked().split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>();
        
        if pinyin_vec.is_empty() { return; }
        
        let new_ex_1a = Exercise1a {
            exercise_id: format!("1a_char_{}", p),
            target_concept_id: format!("char_{}", p),
            prompt: p,
            pinyin: pinyin_vec,
            meaning: meaning.get_untracked(),
            prerequisites: vec![], 
        };
        
        let new_exercise = Exercise::Type1a(new_ex_1a);
        
        set_prompt("".to_string());
        set_pinyin("".to_string());
        set_meaning("".to_string());

        on_save.run((new_exercise,));
    };

    let on_cancel_handler = move |_| {
        set_prompt("".to_string());
        set_pinyin("".to_string());
        set_meaning("".to_string());
        on_cancel.run(());
    };

    view! {
        <form on:submit=on_submit_handler>
            <h3 class="font-bold text-lg dark:text-neutral-content">"Create New Exercise (Type 1a)"</h3>
            
            <div class="form-control w-full py-2">
                 <label class="label">
                    <span class="label-text dark:text-neutral-content">"Prompt (Character)"</span>
                </label>
                <input 
                    type="text"
                    placeholder="你"
                    class="input input-bordered w-full font-mono text-xl"
                    prop:value=move || prompt.get()
                    on:input:target=move |ev| {
                        set_prompt.set(ev.target().value());
                    }
                    required
                />
            </div>

            <div class="form-control w-full py-2">
                <label class="label">
                    <span class="label-text dark:text-neutral-content">"Pinyin (comma-separated)"</span>
                </label>
                <input 
                    type="text" 
                    placeholder="e.g., nǐ"
                    class="input input-bordered w-full"
                    prop:value=move || pinyin.get()
                    on:input:target=move |ev| {
                        set_pinyin.set(ev.target().value());
                    }
                    required
                />
            </div>

            <div class="form-control w-full py-2">
                <label class="label">
                    <span class="label-text dark:text-neutral-content">"Meaning"</span>
                </label>
                <input 
                    type="text" 
                    placeholder="e.g., you"
                    class="input input-bordered w-full"
                    prop:value=move || meaning.get()
                    on:input:target=move |ev| {
                        set_meaning.set(ev.target().value());
                    }
                    required
                />
            </div>

            <div class="modal-action">
                <button type="button" class="btn" on:click=on_cancel_handler>
                    "Cancel"
                </button>
                <button type="submit" class="btn btn-primary">
                    "Create"
                </button>
            </div>
        </form>
    }
}
