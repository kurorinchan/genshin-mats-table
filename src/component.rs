use crate::logic::{
    self, day_to_mat_type, group_by_material, Character, DayOfWeek, TalentLevelUpMaterialType,
};
use leptos::*;
use strum::IntoEnumIterator;

fn display_character_name(characters: &[Character]) -> String {
    characters
        .iter()
        .map(|c| c.name.clone())
        .collect::<Vec<_>>()
        .join(", ")
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
                                <div>
                                {display_character_name(&characters)}
                                </div>
                            }.into_view()
                    }
                })}}
        </Suspense>
        </div>
    }
}

#[component]
fn ShowByDayOfWeek(day_of_week: logic::DayOfWeek) -> impl IntoView {
    let day_to_mat = day_to_mat_type();

    let mat_types = day_to_mat.get(&day_of_week).expect("All days exist");

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
            {day_of_week.as_ref().to_string()}
            </div>

            {mat_views}
        </div>
    }
}

#[component]
pub fn DisplayMats() -> impl IntoView {
    let days = DayOfWeek::iter()
        .map(|day_of_week| {
            view! {
                <div class="col">
                <ShowByDayOfWeek day_of_week={day_of_week} />
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
