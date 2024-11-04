use leptos::*;

use crate::logic::{self, group_by_material, Character};

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
        {mat_type.as_ref().to_string()}
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
pub fn DisplayMats(characters: Vec<logic::Character>) -> impl IntoView {
    characters
        .iter()
        .map(|character| {
            if character.talent_materials.is_empty() {
                return view! {
                    <div>{format!("{} has not talent materials.", character.name)}</div>
                }
                .into_view();
            }
            let material = &character.talent_materials[0];

            view! {
                <MaterialsView mat_type={material.mat_type} />
            }
            .into_view()
        })
        .collect::<Vec<_>>()
}

#[component]
pub fn App() -> impl IntoView {
    let once = create_resource(
        || (),
        |_| async move {
            let characters = logic::read_character_mats();
            let Ok(characters) = characters else {
                return None;
            };
            Some(characters)
        },
    );

    view! {
        <div>
            <div>
                <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                <div>
                    {move || {
                        once.get().map(|mats|{
                            match mats {
                                None => view! {<p>"error!"</p>}.into_view(),
                                Some(mats) => view! {
                                    <DisplayMats characters={mats} />
                                }.into_view(),
                            }
                        })
                    }}
                </div>
                </Suspense>
            </div>
        </div>
    }
}
