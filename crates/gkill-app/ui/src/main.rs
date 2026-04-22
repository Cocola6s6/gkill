mod api;
mod pages;
mod state;

use sycamore::prelude::*;
use wasm_bindgen_futures::spawn_local;
use pages::{installed::InstalledPage, publish::PublishPage, search::SearchPage, settings::SettingsPage};
use state::{AppCtx, Page};

fn main() {
    console_error_panic_hook::set_once();
    sycamore::render(App);
}

#[component]
fn App() -> View {
    let ctx = AppCtx::new();

    {
        let auth = ctx.auth;
        spawn_local(async move {
            if let Ok(status) = api::get_auth_status().await {
                auth.set(status);
            }
        });
    }

    provide_context(ctx.clone());

    view! {
        div(class="flex flex-col h-screen bg-[#f8fafc]") {
            // ── Top Nav ─────────────────────────────────────────────────
            nav(class="flex items-center gap-1 px-5 py-3 bg-white border-b border-[#e4e7ef] shrink-0 shadow-sm") {
                // Logo
                span(class="font-bold text-base mr-5 brand-text") { "gkill" }

                // Tabs
                TabBtn(page=Page::Search,    label="搜索",    icon="🔍")
                TabBtn(page=Page::Installed, label="已安装",  icon="📦")
                TabBtn(page=Page::Publish,   label="发布",    icon="🚀")
                TabBtn(page=Page::Settings,  label="设置",    icon="⚙️")

                // Auth badge
                div(class="ml-auto flex items-center gap-1.5 text-xs") {
                    (if ctx.auth.get_clone().logged_in {
                        let user = ctx.auth.get_clone().user.unwrap_or_default();
                        view! {
                            span(class="w-2 h-2 rounded-full bg-emerald-500 inline-block") {}
                            span(class="text-[#64748b]") { (user) }
                        }
                    } else {
                        view! {
                            span(class="w-2 h-2 rounded-full bg-slate-300 inline-block") {}
                            span(class="text-[#64748b]") { "未登录" }
                        }
                    })
                }
            }

            // ── Page content ────────────────────────────────────────────
            div(class="flex-1 overflow-hidden") {
                (match ctx.page.get() {
                    Page::Search    => view! { SearchPage {} },
                    Page::Installed => view! { InstalledPage {} },
                    Page::Publish   => view! { PublishPage {} },
                    Page::Settings  => view! { SettingsPage {} },
                })
            }

            // ── Toast ────────────────────────────────────────────────────
            (if let Some(msg) = ctx.toast.get_clone() {
                let toast = ctx.toast;
                view! {
                    div(
                        class="fixed bottom-5 right-5 card px-4 py-3 text-sm shadow-lg cursor-pointer max-w-xs z-50 flex items-center gap-2",
                        on:click=move |_| toast.set(None)
                    ) { span { (msg) } }
                }
            } else { view! {} })
        }
    }
}

#[component(inline_props)]
fn TabBtn(page: Page, label: &'static str, icon: &'static str) -> View {
    let ctx = use_context::<AppCtx>();
    let is_active = create_memo(move || ctx.page.get() == page);
    view! {
        button(
            class=format!(
                "flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg text-sm font-medium transition-all {}",
                if is_active.get() {
                    "tab-active shadow-sm"
                } else {
                    "text-[#64748b] hover:bg-[#f1f5f9] hover:text-[#1a2236]"
                }
            ),
            on:click=move |_| ctx.page.set(page)
        ) {
            span { (icon) }
            span { (label) }
        }
    }
}
