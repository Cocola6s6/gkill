use sycamore::prelude::*;
use wasm_bindgen_futures::spawn_local;
use crate::api;
use crate::state::AppCtx;

#[component]
pub fn PublishPage() -> View {
    let ctx = use_context::<AppCtx>();

    let namespace   = create_signal("global".to_string());
    let namespaces: Signal<Vec<String>> = create_signal(vec!["global".to_string()]);
    let visibility  = create_signal("PUBLIC".to_string());
    let path        = create_signal(String::new());
    let busy        = create_signal(false);
    let result: Signal<Option<Result<String, String>>> = create_signal(None);

    // Load user's namespaces on mount
    spawn_local(async move {
        if let Ok(ns_list) = api::get_my_namespaces().await {
            if !ns_list.is_empty() {
                namespace.set(ns_list[0].clone());
                namespaces.set(ns_list);
            }
        }
    });

    view! {
        div(class="flex flex-col h-full bg-[#f8fafc]") {
            div(class="px-6 py-4 bg-white border-b border-[#e4e7ef]") {
                span(class="font-semibold text-[#1a2236]") { "发布 Skill" }
            }

            div(class="flex-1 overflow-y-auto px-6 py-5") {
                div(class="max-w-lg mx-auto space-y-4") {

                    // Namespace
                    div(class="card p-4 space-y-3") {
                        label(class="block text-sm font-semibold text-[#1a2236]") { "命名空间" }
                        div(class="flex flex-wrap gap-2") {
                            Indexed(
                                list=namespaces,
                                view=move |ns_slug| {
                                    let slug = ns_slug.clone();
                                    let slug2 = ns_slug.clone();
                                    view! {
                                        button(
                                            class={
                                                let cur = namespace.get_clone();
                                                if cur == slug {
                                                    "px-3 py-2 rounded-lg text-sm font-medium border-2 border-[#6A6DFF] text-[#6A6DFF] bg-[#f5f5ff]"
                                                } else {
                                                    "px-3 py-2 rounded-lg text-sm font-medium border border-[#e4e7ef] text-[#64748b] hover:bg-[#f8fafc]"
                                                }
                                            },
                                            on:click=move |_| namespace.set(slug2.clone())
                                        ) { (ns_slug) }
                                    }
                                }
                            )
                        }
                    }

                    // Visibility
                    div(class="card p-4 space-y-3") {
                        label(class="block text-sm font-semibold text-[#1a2236]") { "可见性" }
                        div(class="flex gap-2") {
                            button(
                                class={
                                    let v = visibility.get_clone();
                                    if v == "PUBLIC" {
                                        "flex-1 py-2 rounded-lg text-sm font-medium border-2 border-[#6A6DFF] text-[#6A6DFF] bg-[#f5f5ff]"
                                    } else {
                                        "flex-1 py-2 rounded-lg text-sm font-medium border border-[#e4e7ef] text-[#64748b] hover:bg-[#f8fafc]"
                                    }
                                },
                                on:click=move |_| visibility.set("PUBLIC".to_string())
                            ) { "🌍 PUBLIC" }
                            button(
                                class={
                                    let v = visibility.get_clone();
                                    if v == "NAMESPACE_ONLY" {
                                        "flex-1 py-2 rounded-lg text-sm font-medium border-2 border-[#6A6DFF] text-[#6A6DFF] bg-[#f5f5ff]"
                                    } else {
                                        "flex-1 py-2 rounded-lg text-sm font-medium border border-[#e4e7ef] text-[#64748b] hover:bg-[#f8fafc]"
                                    }
                                },
                                on:click=move |_| visibility.set("NAMESPACE_ONLY".to_string())
                            ) { "🔒 NS ONLY" }
                            button(
                                class={
                                    let v = visibility.get_clone();
                                    if v == "PRIVATE" {
                                        "flex-1 py-2 rounded-lg text-sm font-medium border-2 border-[#6A6DFF] text-[#6A6DFF] bg-[#f5f5ff]"
                                    } else {
                                        "flex-1 py-2 rounded-lg text-sm font-medium border border-[#e4e7ef] text-[#64748b] hover:bg-[#f8fafc]"
                                    }
                                },
                                on:click=move |_| visibility.set("PRIVATE".to_string())
                            ) { "🔐 PRIVATE" }
                        }
                        p(class="text-xs text-[#94a3b8]") {
                            (match visibility.get_clone().as_str() {
                                "PUBLIC"         => "所有人可搜索、安装",
                                "NAMESPACE_ONLY" => "仅命名空间成员可见",
                                _                => "仅自己可见",
                            })
                        }
                    }

                    // Path
                    div(class="card p-4 space-y-2") {
                        label(class="block text-sm font-semibold text-[#1a2236]") { "Skill 目录" }
                        div(class="flex gap-2") {
                            input(
                                class="flex-1 font-mono text-sm",
                                placeholder="留空则使用当前目录",
                                bind:value=path
                            )
                            button(
                                class="shrink-0 px-3 py-2 rounded-lg border border-[#e4e7ef] text-sm text-[#64748b] hover:bg-[#f8fafc] font-medium",
                                on:click=move |_| {
                                    spawn_local(async move {
                                        match api::pick_folder().await {
                                            Ok(Some(p)) => path.set(p),
                                            Ok(None) => {}
                                            Err(e) => ctx.toast.set(Some(format!("❌ {e}"))),
                                        }
                                    });
                                }
                            ) { "📂 选择" }
                        }
                        p(class="text-xs text-[#94a3b8]") {
                            "目录内需包含 SKILL.md 文件"
                        }
                    }

                    // Result
                    (if let Some(res) = result.get_clone() {
                        match res {
                            Ok(msg) => view! {
                                div(class="p-3 rounded-lg bg-emerald-50 border border-emerald-200 text-emerald-700 text-sm") {
                                    "✅ " (msg)
                                }
                            },
                            Err(e) => view! {
                                div(class="p-3 rounded-lg bg-red-50 border border-red-200 text-red-600 text-sm") {
                                    "❌ " (e)
                                }
                            },
                        }
                    } else { view! {} })

                    // Submit
                    button(
                        class="w-full btn-primary py-2.5 text-sm font-semibold disabled:opacity-50",
                        disabled=busy.get(),
                        on:click=move |_| {
                            let ns  = namespace.get_clone();
                            let vis = visibility.get_clone();
                            let p   = path.get_clone();
                            busy.set(true);
                            result.set(None);
                            spawn_local(async move {
                                let path_arg = if p.trim().is_empty() { "." } else { p.as_str() };
                                match api::publish_skill(path_arg, &ns, &vis).await {
                                    Ok(_) => {
                                        result.set(Some(Ok(format!("发布成功（{}/{}）", ns, vis))));
                                        ctx.toast.set(Some(format!("✅ 发布成功（{ns}）")));
                                    }
                                    Err(e) => result.set(Some(Err(e))),
                                }
                                busy.set(false);
                            });
                        }
                    ) {
                        (if busy.get() { "发布中…" } else { "发布 Skill" })
                    }
                }
            }
        }
    }
}
