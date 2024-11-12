use crate::logic::{self, day_to_mat_type, group_by_material, relevant_days, Character};
use leptos::*;

#[component]
fn CharacterComponent(character: Character) -> impl IntoView {
    view! {
        <div class="col character-component">
            <div class="border character">
                <div class="text-nowrap character-name">
                    {character.name.clone()}
                </div>
                <img class="border" src={format!("img/{}", &character.thumbnail)} />
            </div>
        </div>
    }
}

#[component]
fn MaterialsView(mat_type: logic::TalentLevelUpMaterialType) -> impl IntoView {
    let characters = create_resource(
        || (),
        move |_| async move {
            let Ok(characters) = logic::read_character_mats() else {
                return vec![];
            };
            let mat_to_characters = group_by_material(characters);
            let characters = mat_to_characters.get(&mat_type).cloned();
            characters.unwrap_or(vec![])
        },
    );

    view! {
        <div>
            <div class="text-warning">
            {mat_type.as_ref().to_string()}
            </div>
        <Suspense
            fallback=move || view! { <p>"Loading..."</p> }
        >
            {move || {
                characters.get().map(|characters| {
                    view! {
                        <div class="container">
                            <div class="row row-cols-4">
                            {characters.iter().map(|character| {
                                view! {
                                    <CharacterComponent character={character.clone()} />
                                }
                            }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }.into_view()
            })}}
        </Suspense>
        </div>
    }
}

#[component]
fn ShowByDayOfWeek(relevant_day: logic::RelevantDay) -> impl IntoView {
    let day_to_mat = day_to_mat_type();

    let mat_types = day_to_mat
        .get(&relevant_day.day_of_week)
        .expect("All days exist");

    let mat_views = mat_types
        .iter()
        .map(|mat_type| {
            view! {
                <div>
                    <MaterialsView mat_type={*mat_type} />
                </div>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div>
            <div class="text-primary fs-3">
            {relevant_day.display_name.clone()}
            </div>

            {mat_views}
        </div>
    }
}

#[component]
pub fn DisplayMats() -> impl IntoView {
    let relevant_days = relevant_days();

    let days = relevant_days
        .iter()
        .map(|day| {
            let today = day.is_today;
            let class_name = move || {
                if today {
                    "dayofweek dayofweek-on border border-primary-subtle"
                } else {
                    "dayofweek dayofweek-off border border-secondary-subtle"
                }
            };
            view! {
                <div class="col" class={class_name}>
                <ShowByDayOfWeek relevant_day={day.clone()} />
                </div>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div class="row row-cols-3">
        {days}
        </div>
    }
}

#[component]
fn TableLegend() -> impl IntoView {
    view! {
        <div>
            <div class="legend-today">
            "本日取れる素材はこの背景色"
            </div>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <h1 class="fs-1">"原神曜日別素材"</h1>
        <TableLegend />
        <div class="container">
            <DisplayMats />
        </div>
    }
}
