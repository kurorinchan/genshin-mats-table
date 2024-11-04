use leptos::*;

use crate::logic;

#[component]
pub fn DisplayMats(characters: Vec<logic::Character>) -> impl IntoView {
    characters
        .iter()
        .map(|character| {
            let name = character.name.clone();
            let talent_mats = &character.talent_materials;
            let talent_mats = talent_mats
                .iter()
                .map(|mat| mat.name.clone())
                .collect::<Vec<String>>()
                .join(", ");
            let content = format!("{}: {}", name, talent_mats);
            view! {
                <div>{content}</div>
            }
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
            <h1>"Hello, World!"</h1>
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
