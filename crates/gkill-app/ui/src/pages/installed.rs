use sycamore::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::state::{AppCtx, InstalledInfo, UpdateCandidate};

#[component]
pub fn InstalledPage() -> View {
    let ctx = use_context::<AppCtx>();

    let list: Signal<Vec<InstalledInfo>>    = create_signal(vec![]);
    let updates: Signal<Vec<UpdateCandidate>> = create_signal(vec![]);
    let checking = create_signal(false);

    let refresh = move || {
        spawn_local(async move {
            if let Ok(items) = api::list_installed().await {
                list.set(items);
            }
        });
    };

    create_effect(move || { refresh(); });

    view! {
        div(class="flex flex-col h-full") {
            // Header
            div(class="flex items-center justify-between px-6 py-4 bg-white border-b border-[#e4e7ef]") {
                div {
                    span(class="font-semibold text-[#1a2236]") { "已安装的 Skill" }
                    (if !list.get_clone().is_empty() {
                        let cnt = list.get_clone().len();
                        view! {
                            span(class="ml-2 text-xs px-2 py-0.5 rounded-full bg-[#f1f5f9] text-[#64748b]") { (cnt) }
                        }
                    } else { view! {} })
                }
                button(
                    class="flex items-center gap-1.5 px-4 py-1.5 rounded-lg border border-[#e4e7ef] text-sm text-[#64748b] hover:bg-[#f8fafc] font-medium",
                    on:click=move |_| {
                        checking.set(true);
                        spawn_local(async move {
                            match api::find_updates().await {
                                Ok(u)  => updates.set(u),
                                Err(e) => ctx.toast.set(Some(format!("❌ {e}"))),
                            }
                            checking.set(false);
                        });
                    }
                ) {
                    (if checking.get() {
                        view! { span(class="animate-spin") { "⟳" } }
                    } else {
                        view! { span { "↑" } }
                    })
                    span { (if checking.get() { "检查中…" } else { "检查更新" }) }
                }
            }

            div(class="flex-1 overflow-y-auto px-6 py-5 space-y-3") {
                // Update banner
                (if !updates.get_clone().is_empty() {
                    view! {
                        div(class="card p-4 border-[#6A6DFF]/30 bg-[#f5f5ff]") {
                            div(class="flex items-center gap-2 text-sm font-semibold text-[#6A6DFF] mb-3") {
                                span { "✦" }
                                span { "可用更新 (" (updates.get_clone().len()) ")" }
                            }
                            Indexed(
                                list=updates,
                                view=move |u: UpdateCandidate| {
                                    let sl     = u.slug.clone();
                                    let ns     = u.namespace.clone();
                                    let sl_disp = sl.clone();
                                    view! {
                                        div(class="flex items-center justify-between py-2 border-b border-[#e4e7ef] last:border-0") {
                                            div {
                                                span(class="text-sm font-medium text-[#1a2236]") { (sl_disp) }
                                                div(class="text-xs text-[#94a3b8] mt-0.5 font-mono") {
                                                    (u.local_published_at) " → " (u.remote_published_at)
                                                }
                                            }
                                            button(
                                                class="btn-primary px-3 py-1 text-xs",
                                                on:click=move |_| {
                                                    let s = sl.clone(); let n = ns.clone();
                                                    spawn_local(async move {
                                                        match api::update_skill(&s, &n).await {
                                                            Ok(_) => { ctx.toast.set(Some(format!("✅ 更新 {s} 完成"))); refresh(); }
                                                            Err(e) => ctx.toast.set(Some(format!("❌ {e}"))),
                                                        }
                                                    });
                                                }
                                            ) { "更新" }
                                        }
                                    }
                                }
                            )
                        }
                    }
                } else { view! {} })

                // Installed list
                (if list.get_clone().is_empty() {
                    view! {
                        div(class="flex flex-col items-center justify-center py-20 text-[#94a3b8]") {
                            span(class="text-4xl mb-3") { "📦" }
                            span(class="text-sm") { "暂无已安装的 skill" }
                        }
                    }
                } else {
                    view! {
                        Indexed(
                            list=list,
                            view=move |item: InstalledInfo| {
                                let sl      = item.slug.clone();
                                let ns      = item.namespace.clone();
                                let sl_disp = sl.clone();
                                let ns_disp = ns.clone();
                                view! {
                                    div(class="card p-4 flex items-center justify-between hover:shadow-md transition-shadow") {
                                        div {
                                            div(class="flex items-center gap-2") {
                                                span(class="font-medium text-sm text-[#1a2236]") { (sl_disp) }
                                                span(class="text-xs px-2 py-0.5 rounded-full bg-[#f1f5f9] text-[#6A6DFF] border border-[#e4e7ef]") {
                                                    (ns_disp)
                                                }
                                            }
                                            div(class="text-xs text-[#94a3b8] mt-1 font-mono") {
                                                "v" (item.version)
                                            }
                                        }
                                        button(
                                            class="px-3 py-1.5 rounded-lg border border-red-200 text-red-500 hover:bg-red-50 text-xs font-medium transition-colors",
                                            on:click=move |_| {
                                                let s = sl.clone(); let n = ns.clone();
                                                spawn_local(async move {
                                                    match api::remove_skill(&s, &n).await {
                                                        Ok(_)  => { ctx.toast.set(Some(format!("🗑️ 已移除 {s}"))); refresh(); }
                                                        Err(e) => ctx.toast.set(Some(format!("❌ {e}"))),
                                                    }
                                                });
                                            }
                                        ) { "移除" }
                                    }
                                }
                            }
                        )
                    }
                })
            }
        }
    }
}
