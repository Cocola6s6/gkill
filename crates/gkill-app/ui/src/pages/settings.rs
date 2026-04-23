use sycamore::prelude::*;
use wasm_bindgen_futures::spawn_local;
use std::cell::Cell;
use std::rc::Rc;
use crate::api;
use crate::state::AppCtx;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[component]
pub fn SettingsPage() -> View {
    let ctx = use_context::<AppCtx>();

    let token   = create_signal(String::new());
    let busy    = create_signal(false);
    let err: Signal<Option<String>> = create_signal(None);
    let alive = Rc::new(Cell::new(true));

    on_cleanup({
        let alive = alive.clone();
        move || alive.set(false)
    });

    view! {
        div(class="flex flex-col h-full bg-[#f8fafc]") {
            div(class="px-6 py-4 bg-white border-b border-[#e4e7ef]") {
                span(class="font-semibold text-[#1a2236]") { "设置" }
            }

            div(class="flex-1 overflow-y-auto px-6 py-5 space-y-4 max-w-lg mx-auto w-full") {

                // Auth status card
                div(class="card p-4") {
                    (if ctx.auth.get_clone().logged_in {
                        let user     = ctx.auth.get_clone().user.unwrap_or_else(|| "已登录".into());
                        let registry = ctx.auth.get_clone().registry;
                        view! {
                            div(class="flex items-center gap-3 mb-4") {
                                div(class="w-10 h-10 rounded-full flex items-center justify-center text-lg",
                                    style="background: linear-gradient(135deg, #6A6DFF 0%, #B85EFF 100%);") {
                                    "👤"
                                }
                                div {
                                    div(class="font-semibold text-sm text-[#1a2236]") { (user) }
                                    div(class="text-xs text-[#94a3b8] font-mono truncate max-w-[220px]") { (registry) }
                                }
                            }
                            button(
                                class="w-full py-2 rounded-lg border border-red-200 text-red-500 hover:bg-red-50 text-sm font-medium transition-colors disabled:opacity-50",
                                disabled=busy.get(),
                                on:click={
                                    let alive = alive.clone();
                                    move |_| {
                                        let alive = alive.clone();
                                        busy.set(true);
                                        spawn_local(async move {
                                            match api::logout().await {
                                                Ok(_) => {
                                                    ctx.toast.set(Some("✅ 已退出登录".into()));
                                                    if let Ok(s) = api::get_auth_status().await {
                                                        if alive.get() {
                                                            ctx.auth.set(s);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    if alive.get() {
                                                        err.set(Some(e));
                                                    }
                                                }
                                            }
                                            if alive.get() {
                                                busy.set(false);
                                            }
                                        });
                                    }
                                }
                            ) { (if busy.get() { "退出中…" } else { "退出登录" }) }
                        }
                    } else {
                        view! {
                            div(class="flex items-center gap-3 mb-4") {
                                div(class="w-10 h-10 rounded-full bg-[#f1f5f9] flex items-center justify-center text-lg border border-[#e4e7ef]") {
                                    "🔒"
                                }
                                div {
                                    div(class="font-semibold text-sm text-[#1a2236]") { "未登录" }
                                    div(class="text-xs text-[#94a3b8]") { "登录后可发布和管理 skill" }
                                }
                            }

                            // Error
                            (if let Some(e) = err.get_clone() {
                                view! {
                                    div(class="mb-3 p-2.5 rounded-lg bg-red-50 border border-red-200 text-red-600 text-xs") {
                                        (e)
                                    }
                                }
                            } else { view! {} })

                            // Token input
                            div(class="space-y-2") {
                                label(class="block text-sm font-medium text-[#64748b]") { "Token" }
                                input(
                                    class="w-full font-mono text-sm",
                                    r#type="password",
                                    placeholder="粘贴 Token…",
                                    bind:value=token
                                )
                            }
                            button(
                                class="w-full btn-primary py-2.5 mt-3 text-sm font-semibold disabled:opacity-50",
                                disabled=busy.get() || token.get_clone().trim().is_empty(),
                                on:click={
                                    let alive = alive.clone();
                                    move |_| {
                                        let t = token.get_clone();
                                        let alive = alive.clone();
                                        busy.set(true);
                                        err.set(None);
                                        spawn_local(async move {
                                            match api::login(&t).await {
                                                Ok(_) => {
                                                    ctx.toast.set(Some("✅ 登录成功".into()));
                                                    if alive.get() {
                                                        token.set(String::new());
                                                    }
                                                    if let Ok(s) = api::get_auth_status().await {
                                                        if alive.get() {
                                                            ctx.auth.set(s);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    if alive.get() {
                                                        err.set(Some(e));
                                                    }
                                                }
                                            }
                                            if alive.get() {
                                                busy.set(false);
                                            }
                                        });
                                    }
                                }
                            ) { (if busy.get() { "验证中…" } else { "登录" }) }
                        }
                    })
                }

                // About
                div(class="card p-4 space-y-2") {
                    p(class="text-xs font-semibold text-[#1a2236] mb-2") { "关于" }
                    div(class="text-xs text-[#64748b] space-y-1") {
                        div(class="flex justify-between") {
                            span { "版本" }
                            span(class="font-mono text-[#1a2236]") { (VERSION) }
                        }
                        div(class="flex justify-between") {
                            span { "Registry" }
                            span(class="font-mono text-[#1a2236] truncate max-w-[200px]") { (ctx.auth.get_clone().registry) }
                        }
                    }
                }
            }
        }
    }
}
