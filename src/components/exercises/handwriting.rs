use std::time::Duration;

use leptos::either::either;
use leptos::ev::PointerEvent;
use leptos::html::Div;
use leptos::logging::log;
use leptos::prelude::*;

use crate::logic::stroke_matching::matcher::StrokeMatchResult;
use crate::logic::stroke_matching::{self, models};
use crate::model::error::ArcZhongCharError;
use crate::model::hanzi_strokes::HanziStrokes;

// A helper to build an SVG path string from points.
fn points_to_svg_path(points: &[models::Point]) -> String {
    if points.is_empty() {
        return String::new();
    }
    let start = format!("M {} {}", points[0].x, points[0].y);
    let rest: String = points.iter().skip(1).map(|p| format!("L {} {}", p.x, p.y)).collect();
    format!("{} {}", start, rest)
}

fn medians_to_svg_path(points: &[[f64; 2]]) -> String {
    if points.is_empty() {
        return String::new();
    }
    let start = format!("M {} {}", points[0][0], points[0][1]);
    let rest: String = points.iter().skip(1).map(|p| format!("L {} {}", p[0], p[1])).collect();
    format!("{} {}", start, rest)
}

fn calculate_stroke_len(points: &[[f64; 2]]) -> f64 {
    if points.len() < 2 {
        return 0.0;
    }
    let mut total_len = points.windows(2).map(|p_pair| {
        let p1 = p_pair[0];
        let p2 = p_pair[1];
        ((p2[0] - p1[0]).powi(2) + (p2[1] - p1[1]).powi(2)).sqrt()
    }).sum();

    // From HanziWriter's source, an offset is added for animation smoothness.
    const STROKE_WIDTH_FOR_ANIM: f64 = 200.0;
    total_len += STROKE_WIDTH_FOR_ANIM / 2.0;
    total_len
}


fn strip_duplicate_points(points: Vec<models::Point>) -> Vec<models::Point> {
    let mut deduped = Vec::new();
    if let Some(first) = points.first() {
        deduped.push(*first);
        for point in points.iter().skip(1) {
            if let Some(last) = deduped.last() {
                if last != point {
                    deduped.push(*point);
                }
            }
        }
    }
    deduped
}



#[component]
pub fn HandwritingExercise(
    #[prop(into)] character: Signal<char>,
    #[prop(into)] on_complete: Callback<()>,
) -> impl IntoView {
    let strokes_resource = LocalResource::new(move || async move {
        let char_val = character.get();
        HanziStrokes::get_one_from_db(char_val)
            .await
            .map_err(|e| ArcZhongCharError::from(e))
    });

    // --- State for drawing ---
    let (is_drawing, set_is_drawing) = signal(false);
    let (user_points, set_user_points) = signal(Vec::<models::Point>::new());
    
    //let user_path = Signal::derive(move || points_to_svg_path(&user_points.get()));
    let user_drawing_path = Signal::derive(move || {
        let points = user_points.get();
        if points.is_empty() { return String::new(); }
        let start = format!("M {} {}", points[0].x, points[0].y);
        let rest: String = points.iter().skip(1).map(|p| format!("L {} {}", p.x, p.y)).collect();
        format!("{} {}", start, rest)
    });


    let drawing_area_ref = NodeRef::<Div>::new();
    
    // --- State for the quiz ---
    let (current_stroke_index, set_current_stroke_index) = signal(0);
    

    let (show_hint, set_show_hint) = signal(false);
    let (hint_clip_path_d, set_hint_clip_path_d) = signal(String::new()); // The "stencil"
    let (hint_median_path_d, set_hint_median_path_d) = signal(String::new()); // The "paint" path
    let (hint_stroke_len, set_hint_stroke_len) = signal(0.0);

    // --- Coordinate Scaling ---
    // The div is 288px (72 * 4), but the SVG is 1024x1024. We need to scale pointer events.
    const SCALE_FACTOR: f64 = 1024.0 / 288.0;

    let on_pointer_down = move |ev: PointerEvent| {
        if let Some(el) = drawing_area_ref.get() {
            // This tells the browser to send all subsequent pointer events to this element.
            _ = el.set_pointer_capture(ev.pointer_id());
        }

        if show_hint.get() { return; }
        
        set_is_drawing.set(true);
        set_user_points.set(Vec::new());
        // Apply scaling to the coordinates
        let pt = models::Point {
            x: ev.offset_x() as f64 * SCALE_FACTOR,
            y: 1024.0 - (ev.offset_y() as f64 * SCALE_FACTOR), //ev.offset_y() as f64 * SCALE_FACTOR,
        };
        set_user_points.update(|points| points.push(pt));
    };

    let on_pointer_move = move |ev: PointerEvent| {
        if is_drawing.get() {
            // Apply scaling to the coordinates
            let pt = models::Point {
                x: ev.offset_x() as f64 * SCALE_FACTOR,
                y: 1024.0 - (ev.offset_y() as f64 * SCALE_FACTOR),
            };
            set_user_points.update(|points| points.push(pt));
        }
    };

    let on_pointer_up = move |_ev: PointerEvent| {
        set_is_drawing.set(false);
        let points = strip_duplicate_points(user_points.get_untracked());
        if points.len() < 2 {
            set_user_points.set(Vec::new());
            return;
        }
        let user_stroke = models::UserStroke { points };

        if let Some(Ok(Some(hanzi_strokes))) = strokes_resource.get() {
            if let Some(correct_stroke_medians_vec) = hanzi_strokes.medians.get(current_stroke_index.get())
            {
                let correct_stroke_medians: Vec<[f64; 2]> = correct_stroke_medians_vec
                    .iter()
                    .map(|p| [p[0], p[1]])
                    .collect();

                let correct_stroke_points: Vec<models::Point> = correct_stroke_medians
                    .iter()
                    .map(|p| models::Point { x: p[0], y: p[1] })
                    .collect();

                let correct_stroke = models::Stroke::new(correct_stroke_points);

                let result: StrokeMatchResult =
                    stroke_matching::matcher::stroke_matches(&user_stroke, &correct_stroke);

                if result.is_match {
                    let new_stroke_index = current_stroke_index.get_untracked() + 1;

                    if new_stroke_index == hanzi_strokes.strokes.len() {
                        on_complete.run(());
                    } else {
                        set_current_stroke_index.set(new_stroke_index);
                        set_user_points.set(Vec::new());
                    }
                } else {
                    let clip_path_d = hanzi_strokes.strokes[current_stroke_index.get()].clone();
                    
                    // 2. Get data for the ANIMATED PATH (the stroke centerline "paint").
                    let median_path_d = medians_to_svg_path(&correct_stroke_medians);
                    let stroke_len = calculate_stroke_len(&correct_stroke_medians);

                    // 3. Set signals to trigger the animation.
                    set_hint_clip_path_d.set(clip_path_d);
                    set_hint_median_path_d.set(median_path_d);
                    set_hint_stroke_len.set(stroke_len);
                    set_show_hint.set(true);

                    // 4. Set timeout to hide hint and clear drawing.
                    set_timeout(
                        move || {
                            set_show_hint.set(false);
                            set_user_points.set(Vec::new());
                        },
                        // Animation is 1s, give a little buffer before clearing.
                        Duration::from_millis(1200),
                    );
                    // set_timeout(
                    //     move || {
                    //         set_user_points.set(Vec::new());
                    //     },
                    //     Duration::from_millis(500),
                    // );
                }
            }
        }
    };

    view! {
        <div class="flex flex-col items-center">
            <h2 class="card-title mb-4">"Please write the character."</h2>
            <div
                class="w-72 h-72 bg-base-200 rounded-lg touch-none"
                node_ref=drawing_area_ref
                on:pointerdown=on_pointer_down
                on:pointermove=on_pointer_move
                on:pointerup=on_pointer_up
            >
                <Suspense fallback=move || view!{ <div class="skeleton w-full h-full"></div> }>
                    {Suspend::new(async move {
                        let res = strokes_resource.await;
                        either!(res,
                            Ok(Some(strokes)) => {
                                let local_strokes = strokes.clone();
                                let local_strokes2 = local_strokes.clone();
                                view! {
                                <svg viewBox="0 0 1024 1024" xmlns="http://www.w3.org/2000/svg">
                                    <g class="guidelines" stroke="currentColor" opacity="1.0">
                                        // Crosshairs
                                        <line x1="512" y1="0" x2="512" y2="1024" stroke-width="12" stroke-dasharray="20, 20">></line>
                                        <line x1="0" y1="512" x2="1024" y2="512" stroke-width="12" stroke-dasharray="20, 20">></line>
                                        
                                    </g>
                                    <defs>
                                        {move || either!(show_hint.get(),
                                            true => {
                                                let clip_id = format!("clip-{}", current_stroke_index.get());
                                                view! {
                                                    <clipPath id=clip_id>
                                                        <path d=move || hint_clip_path_d.get()></path>
                                                    </clipPath>
                                                }
                                            },
                                            false => ().into_view()
                                        )}
                                    </defs>

                                    // This group has the inverted Y-axis for the character data
                                    <g transform="scale(1,-1) translate(0, -900)">
                                        {move || {
                                            let completed = (0..current_stroke_index.get()).map(|i| {
                                                let d = local_strokes.strokes[i].clone();
                                                view! { <path fill="currentColor" opacity="1.0" d=d></path> }
                                            }).collect::<Vec<_>>();
                                            view! {
                                                <>
                                                    {completed}
                                                </>
                                            }
                                        }}
                                    </g>
                                    // <g transform="scale(1, -1) translate(0, -900)" opacity="0.2">
                                    //     {move || {
                                    //         let remaining = (current_stroke_index.get()..local_strokes2.strokes.len()).map(|i| {
                                    //             let d = local_strokes2.strokes[i].clone();
                                    //             view! { <path fill="currentColor" d=d></path> }
                                    //         }).collect::<Vec<_>>();
                                            
                                    //         view! {
                                    //             <>
                                    //                 {remaining}
                                    //             </>
                                    //         }
                                    //     }}
                                    // </g>
                                    <g transform="scale(1, -1) translate(0, -1024)">
                                            <path
                                            d=move || user_drawing_path.get()
                                            fill="none"
                                            stroke="var(--color-primary)"
                                            stroke-width="30"
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                        />
                                    </g>
                                    <g transform="scale(1, -1) translate(0, -900)">
                                        {move || {
                                            view! {
                                                <>
                                                {move || either!(show_hint.get(),
                                                        true => {
                                                            let clip_url = format!("url(#clip-{})", current_stroke_index.get());
                                                            let len = hint_stroke_len.get();
                                                            log!("clip_url: {} len: {}", clip_url, len);
                                                            view! {
                                                                // This is the thick, animated "paint" line
                                                                <path
                                                                    class="animate-stroke-hint"
                                                                    clip-path=clip_url
                                                                    d=move || hint_median_path_d.get()
                                                                    fill="none"
                                                                    stroke="currentColor"
                                                                    stroke-width="200" // Must be thick enough to fill the stencil
                                                                    stroke-linecap="round"
                                                                    stroke-linejoin="round"
                                                                    style:stroke-dasharray=move || format!("{}", len)
                                                                    style:stroke-dashoffset=move || format!("{}", len)
                                                                />
                                                            }
                                                        },
                                                        false => ().into_view()
                                                    )}
                                                </>
                                            }
                                        }}
                                    </g>
                                </svg>
                            }}.into_view(),
                            _ => view! { <p>"Could not load character data."</p> }.into_view()
                        )
                    })}
                </Suspense>
            </div>
        </div>
    }
}

