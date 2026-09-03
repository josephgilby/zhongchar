use leptos::prelude::*;
use leptos_meta::*;
use crate::components::nav::Nav;
use crate::model::error::ArcZhongCharError;
use crate::model::radical::Radical;

#[component]
pub fn Radicals() -> impl IntoView {
    let radicals_fetched = LocalResource::new(move || async move {
        Radical::get_all_from_db()
            .await
            .map_err(|e| ArcZhongCharError::from(e))
    });
    view! {
        <main>
            <Title text="中 Char"/>
            <Nav/>
            <div class="w-full h-dvh absolute top-0 flex flex-col">
                <div class="w-full h-16 flex-none"></div>
                <div id="radical-table-container" class="w-full flex-grow overflow-x-auto overflow-y-auto">
                <table class="table table-zebra table-pin-rows w-full">
                    <thead>
                        <tr>
                            <th>"Number"</th>
                            <th>"Radical Forms"</th>
                            <th>"Stroke Count"</th>
                            <th>"Meaning"</th>
                            <th>"Colloquial Term"</th>
                            <th>"Pinyin"</th>
                            <th>"Han Viet"</th>
                            <th>"Hiragana/Romaji"</th>
                            <th>"Hangul/Romaja"</th>
                            <th>"Frequency"</th>
                            <th>"Simplified"</th>
                            <th>"Examples"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <Suspense fallback=move || {
                            let vec_20: Vec<i32> = (0..20).collect();
                            vec_20.into_iter().map(|n| {
                                if n % 2 == 0 {
                                    view!{<tr><td colspan="12">"\u{00A0}"</td></tr>}
                                } else {
                                    view!{<tr><td colspan="12" class="skeleton">"\u{00A0}"</td></tr>}
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
                                }
                            >
                                { move || Suspend::new ( async move {
                                    radicals_fetched.await.map(|radicals| { // Changed this line
                                        view! {
                                            <> // Added Fragment here
                                                {
                                                    radicals.into_iter().map(|radical| {
                                                        view! {
                                                            <tr>
                                                                <td>{radical.number}</td>
                                                                <td>{radical.radical_forms}</td>
                                                                <td>{radical.stroke_count}</td>
                                                                <td>{radical.meaning}</td>
                                                                <td>{radical.colloquial_term}</td>
                                                                <td>{radical.pinyin}</td>
                                                                <td>{radical.han_viet}</td>
                                                                <td>{radical.hiragana_romaji}</td>
                                                                <td>{radical.hangul_romaja}</td>
                                                                <td>{radical.frequency}</td>
                                                                <td>{radical.simplified}</td>
                                                                <td>{radical.examples}</td>
                                                            </tr>
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
        </main>
    }

}