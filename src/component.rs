use crate::logic::{self, day_to_mat_type, group_by_material, relevant_days, Character};
use leptos::*;

#[component]
fn CharacterComponent(character: Character) -> impl IntoView {
    view! {
        <div class="col border character">
                <div class="text-nowrap character-name">
                    {character.name.clone()}
                </div>
                <img class="character border" src={format!("img/{}", &character.thumbnail)} />
        </div>
    }
}

#[component]
fn MaterialsView(mat_type: logic::TalentLevelUpMaterialType) -> impl IntoView {
    let characters = create_resource(
        || (),
        move |_| async move {
            let characters = logic::read_character_mats().ok()?;
            let mat_to_characters = group_by_material(characters);
            mat_to_characters.get(&mat_type).cloned()
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
                    match characters {
                        None => view! {<p>"error!"</p>}.into_view(),
                        Some(characters) =>
                            view! {
                                <div class="container">
                                    <div class="row">
                                    {characters.iter().map(|character| {
                                        view! {
                                            <CharacterComponent character={character.clone()} />
                                        }
                                    }).collect::<Vec<_>>()}
                                    </div>
                                </div>
                            }.into_view()
                    }
                })}}
        </Suspense>
        </div>
    }
}

#[component]
fn ShowByDayOfWeek(relevant_days: logic::RelevantDays) -> impl IntoView {
    let day_to_mat = day_to_mat_type();

    let mat_types = day_to_mat
        .get(&relevant_days.day_of_week)
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
            <div class="text-primary">
            {relevant_days.display_name.clone()}
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
            view! {
                <div class="col">
                <ShowByDayOfWeek relevant_days={day.clone()} />
                </div>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div class="row">
        {days}
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    view! {
        <div class="container">
            <DisplayMats />
        </div>
    }
}
