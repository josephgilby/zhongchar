use std::ops::Range;

use leptos::logging::log;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_struct_table::{EventHandler, TableContent, TableDataProvider, TableRow};
use serde::{Deserialize, Serialize};
use crate::components::nav::Nav;
use crate::model::error::ArcZhongCharError;
use crate::model::hanzi::Hanzi;

#[derive(Debug, Serialize, Deserialize, Clone, TableRow)]
#[table(sortable)] // Make the table sortable
pub struct HanziTableElement {
    #[table(title = "Character")]
    pub character: char,
    #[table(title = "Definition", renderer = "OptionalTruncatedCellRenderer")]
    pub definition: Option<String>,
    #[table(title = "Pinyin", renderer = "TruncatedCellRenderer")]
    pub pinyin: String,
    #[table(title = "Decomposition", renderer = "TruncatedCellRenderer")]
    pub decomposition: String,
    #[table(title = "Radical", renderer = "TruncatedCellRenderer")]
    pub radical: String,
    #[table(title = "Ety. Type", renderer = "TruncatedCellRenderer")]
    pub r#type: String,
    #[table(title = "Ety. Hint", renderer = "TruncatedCellRenderer")]
    pub hint: String,
    #[table(title = "Ety. Phonetic", renderer = "OptionalTruncatedCellRenderer")]
    pub phonetic: Option<String>,
    #[table(title = "Ety. Semantic", renderer = "OptionalTruncatedCellRenderer")]
    pub semantic: Option<String>,
    #[table(title = "Matches", renderer = "TruncatedCellRenderer")]
    pub matches: String,
}

#[component]
fn OptionalTruncatedCellRenderer(
    class: String,
    #[prop(into)] value: Signal<Option<String>>,
    row: RwSignal<HanziTableElement>,
    index: usize,
) -> impl IntoView {
    view! {
        <td class=format!("{class} h-12 p-2")>
            {move || value.get().map(|text| {
                let text_cloned = text.clone();
                view! {
                    <div
                        class="tooltip tooltip-align-end"
                        data-tip=text_cloned
                    >
                        // This div now only contains the text and the line-clamp styles.
                        <div class="line-clamp-2 break-all text-left">
                            {text}
                        </div>
                    </div>
                }
            })}
            // <div class="h-12 p-1 line-clamp-2 break-all">
            //     {value}
            // </div>
        </td>
    }
}


#[component]
fn TruncatedCellRenderer(
    class: String,
    #[prop(into)] value: Signal<String>,
    row: RwSignal<HanziTableElement>,
    index: usize,
) -> impl IntoView {
    view! {
        <td class=format!("{class} h-12 p-2")>
            <div class="tooltip tooltip-align-end" data-tip=move || value.get()>
                <div class="line-clamp-2 break-all text-left">
                    {value}
                </div>
            </div>
        </td>
    }
}

pub fn StripedTableRowRenderer(
    class: Signal<String>,
    row: RwSignal<HanziTableElement>,
    index: usize,
    selected: Signal<bool>,
    on_select: EventHandler<web_sys::MouseEvent>,
) -> impl IntoView {
    view! {
        <tr class=class class:bg-base-100={index % 2 == 0} on:click=move |mouse_event| on_select.run(mouse_event)>
            {TableRow::render_row(row, index)}
        </tr>
    }

    // view! {
    //     <g
    //         class=class
    //         transform=transform
    //         on:click=move |mouse_event| on_select.run(mouse_event)
    //     >
    //         <line
    //             x1="5"
    //             y1="0"
    //             x2="150"
    //             y2="0"
    //             stroke-width="1px"
    //             stroke="black"
    //             opacity="0.1"
    //         ></line>
    //         {TableRow::render_row(row, index)}
    //     </g>
    // }
}


#[derive(Default, Clone, Copy)]
pub struct HanziTableDataProvider;

impl TableDataProvider<HanziTableElement> for HanziTableDataProvider {
    async fn get_rows(&self, range: Range<usize>) -> Result<(Vec<HanziTableElement>, Range<usize>), String> {
        // Call our database function
        let hanzi_rows = Hanzi::get_range(range.clone())
            .await
            .map_err(|e| e.to_string())?;

        let range_len = hanzi_rows.len();

        // 2. Transform the data into the `HanziTableElement` format
        let transformed_rows = hanzi_rows.into_iter().map(HanziTableElement::from).collect();

        // 3. Return the transformed data and the correct range
        Ok((transformed_rows, range.start..(range.start + range_len)))
    }

    async fn row_count(&self) -> Option<usize> {
        // Call our database function
        Hanzi::get_count().await.ok()
    }
}

impl From<Hanzi> for HanziTableElement {
    fn from(hanzi: Hanzi) -> Self {
        // Use the existing Etymology struct or a default if it's None
        let etymology = hanzi.etymology.unwrap_or_default();

        Self {
            character: hanzi.character,
            definition: hanzi.definition,
            pinyin: hanzi.pinyin.join(", "), // Flatten the Vec<String>
            decomposition: hanzi.decomposition,
            radical: hanzi.radical,
            r#type: etymology.r#type,
            hint: etymology.hint,
            phonetic: etymology.phonetic,
            semantic: etymology.semantic,
            matches: format!("{:?}", hanzi.matches), // Format the Vec for display
        }
    }
}


#[component]
pub fn Home() -> impl IntoView {
    let scroll_container = NodeRef::new();
    let rows = HanziTableDataProvider::default();
    view! {
        <main>
            <Title text="中 Char"/>
            <Nav/>
            <div class="w-full h-dvh absolute top-0 flex flex-col">
                <div class="w-full h-16 flex-none bg-base-300"></div>
                <p class="hidden">{format!("{:#?}", scroll_container)}</p>
                <div id="hanzi-table-container" node_ref=scroll_container class="w-full flex-grow overflow-x-auto overflow-y-auto">
                    <table class="table table-pin-rows">
                        <TableContent
                            rows
                            scroll_container
                            row_renderer=StripedTableRowRenderer
                        />
                    </table>
                </div>
            </div>
        </main>
    }
}
