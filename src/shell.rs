use crate::prelude::*;

mod nav;
mod footer;

/// Provides a layout shell for pages.
#[component]
pub fn Shell(children: Children) -> impl IntoView {
    view! {
        <div class="grid grid-cols-[20rem_1fr] font-sans subpixel-antialiased bg-stone-700 text-stone-300">
            <nav::Nav />
            <div class="flex flex-col">
                <main>
                    {children()}
                </main>
                <div class="grow" />
                <footer::Footer />
            </div>
        </div>
    }
}
