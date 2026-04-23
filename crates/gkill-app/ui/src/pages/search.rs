use sycamore::prelude::*;
use wasm_bindgen_futures::spawn_local;
use std::cell::Cell;
use std::rc::Rc;
use crate::api;
use crate::state::{AppCtx, SkillItem, SkillPage};

#[component]
pub fn SearchPage() -> View {
    let ctx = use_context::<AppCtx>();

    let query    = create_signal(String::new());
    let page_no  = create_signal(0u32);
    let result: Signal<Option<SkillPage>> = create_signal(None);
    let loading  = create_signal(false);
    let initialized = create_signal(false);
    let error: Signal<Option<String>> = create_signal(None);
    let detail_open = create_signal(false);
    let detail_title = create_signal(String::new());
    let detail_loading = create_signal(false);
    let detail_error: Signal<Option<String>> = create_signal(None);
    let detail_html: Signal<Option<String>> = create_signal(None);
    let alive = Rc::new(Cell::new(true));

    on_cleanup({
        let alive = alive.clone();
        move || alive.set(false)
    });

    let do_search: Rc<dyn Fn()> = {
        let alive = alive.clone();
        Rc::new(move || {
            let q  = query.get_clone();
            let pg = page_no.get();
            let alive = alive.clone();
            loading.set(true);
            error.set(None);
            spawn_local(async move {
                let resp = api::search_skills(&q, "relevance", pg, 12).await;
                if !alive.get() {
                    return;
                }
                match resp {
                    Ok(res) => result.set(Some(res)),
                    Err(e)  => error.set(Some(e)),
                }
                loading.set(false);
            });
        })
    };

    {
        let do_search = do_search.clone();
        create_effect(move || {
            if !initialized.get() {
                initialized.set(true);
                (do_search)();
            }
        });
    }

    view! {
        div(class="flex flex-col h-full") {
            // Search bar
            div(class="flex gap-2.5 px-6 py-4 bg-white border-b border-[#e4e7ef]") {
                div(class="relative flex-1") {
                    span(class="absolute left-3 top-1/2 -translate-y-1/2 text-[#94a3b8] text-sm pointer-events-none") { "🔍" }
                    input(
                        class="w-full pl-9 pr-4 py-2 text-sm",
                        placeholder="搜索 skill 名称、描述…",
                        bind:value=query,
                        on:keydown={
                            let do_search = do_search.clone();
                            move |e: web_sys::KeyboardEvent| {
                                if e.key() == "Enter" { page_no.set(0); (do_search)(); }
                            }
                        }
                    )
                }
                button(
                    class="btn-primary px-5 py-2 text-sm shrink-0",
                    on:click={
                        let do_search = do_search.clone();
                        move |_| { page_no.set(0); (do_search)(); }
                    }
                ) {
                    (if loading.get() {
                        view! { "搜索中…" }
                    } else {
                        view! { "搜索" }
                    })
                }
            }

            // Status
            (if let Some(e) = error.get_clone() {
                view! {
                    div(class="mx-6 mt-4 p-3 rounded-lg bg-red-50 border border-red-200 text-red-600 text-sm") { (e) }
                }
            } else { view! {} })

            // Results grid
            div(class="flex-1 overflow-y-auto px-6 py-5") {
                (if let Some(res) = result.get_clone() {
                    let total = res.total;
                    let items_sig = create_signal(res.items);
                    let alive_for_items = alive.clone();
                    view! {
                        // Result count
                        div(class="text-xs text-[#94a3b8] mb-4 flex items-center gap-2") {
                            "共 " (total) " 个结果"
                            (if loading.get() {
                                view! {
                                    span(class="inline-flex items-center gap-1.5") {
                                        span(class="animate-spin") { "⟳" }
                                        span { "刷新中…" }
                                    }
                                }
                            } else { view! {} })
                        }
                        div(class="grid grid-cols-1 gap-3") {
                            Indexed(
                                list=items_sig,
                                view=move |item: SkillItem| {
                                    let slug      = item.slug.clone();
                                    let ns        = item.namespace.clone().unwrap_or_else(|| "global".into());
                                    let name      = item.display_name.clone().unwrap_or_else(|| slug.clone());
                                    let desc      = item.summary.clone().unwrap_or_default();
                                    let dl        = item.download_count.unwrap_or(0);
                                    let agent     = ctx.agent.get_clone();
                                    let mode      = ctx.mode.get_clone();
                                    let toast     = ctx.toast;
                                    let sl_btn    = slug.clone();
                                    let ns_btn    = ns.clone();
                                    let sl_detail = slug.clone();
                                    let ns_detail = ns.clone();
                                    let title_detail = name.clone();
                                    let alive_for_detail = alive_for_items.clone();
                                    view! {
                                        div(
                                            class="card p-4 flex items-start justify-between gap-4 hover:shadow-md transition-shadow cursor-pointer",
                                            on:click=move |_| {
                                                let s = sl_detail.clone();
                                                let n = ns_detail.clone();
                                                let title = title_detail.clone();
                                                let alive = alive_for_detail.clone();
                                                detail_title.set(title);
                                                detail_open.set(true);
                                                detail_loading.set(true);
                                                detail_error.set(None);
                                                detail_html.set(None);
                                                spawn_local(async move {
                                                    let resp = api::get_skill_markdown(&s, &n, None).await;
                                                    if !alive.get() {
                                                        return;
                                                    }
                                                    match resp {
                                                        Ok(md) => detail_html.set(Some(api::render_markdown(&md))),
                                                        Err(e) => detail_error.set(Some(e)),
                                                    }
                                                    detail_loading.set(false);
                                                });
                                            }
                                        ) {
                                            div(class="flex-1 min-w-0") {
                                                div(class="flex items-center gap-2 mb-1") {
                                                    span(class="font-semibold text-sm text-[#1a2236] truncate") { (name) }
                                                    span(class="shrink-0 text-xs px-2 py-0.5 rounded-full bg-[#f1f5f9] text-[#6A6DFF] font-medium border border-[#e4e7ef]") {
                                                        (ns)
                                                    }
                                                }
                                                (if !desc.is_empty() {
                                                    let d = desc.clone();
                                                    view! {
                                                        p(class="text-xs text-[#64748b] line-clamp-2 leading-relaxed") { (d) }
                                                    }
                                                } else { view! {} })
                                                div(class="mt-2 flex items-center gap-3 text-xs text-[#94a3b8]") {
                                                    span(class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-[#f8fafc] border border-[#e2e8f0] text-[#334155] font-semibold text-[11px]") {
                                                        span(class="inline-flex items-center justify-center w-4 h-4 rounded-full bg-white border border-[#e2e8f0]") {
                                                            span(class="text-[10px] leading-none") { "✅" }
                                                        }
                                                        span { (dl) }
                                                    }
                                                    span(class="font-mono text-[11px]") { (slug) }
                                                }
                                            }
                                            button(
                                                class="btn-primary shrink-0 px-4 py-1.5 text-xs",
                                                on:click=move |e: web_sys::MouseEvent| {
                                                    e.stop_propagation();
                                                    let s = sl_btn.clone();
                                                    let n = ns_btn.clone();
                                                    let ag = agent.clone();
                                                    let mo = mode.clone();
                                                    spawn_local(async move {
                                                        match api::install_skill(&s, &n, &ag, &mo).await {
                                                            Ok(_) => toast.set(Some(format!("✅ 已安装 {s}"))),
                                                            Err(e) => toast.set(Some(format!("❌ {e}"))),
                                                        }
                                                    });
                                                }
                                            ) { "安装" }
                                        }
                                    }
                                }
                            )
                        }
                    }
                } else { view! {} })
            }

            // Pagination
            (if let Some(res) = result.get_clone() {
                let total = res.total;
                let size  = res.size as u64;
                let total_pages = ((total + size - 1) / size).max(1);
                let cur = page_no.get() as u64;
                view! {
                    div(class="flex items-center justify-center gap-4 px-6 py-3 bg-white border-t border-[#e4e7ef] text-sm") {
                        button(
                            class="px-3 py-1.5 rounded-lg border border-[#e4e7ef] text-[#64748b] hover:bg-[#f8fafc] disabled:opacity-40 text-xs font-medium",
                            disabled=cur == 0,
                            on:click={
                                let do_search = do_search.clone();
                                move |_| {
                                    let p = page_no.get();
                                    if p > 0 { page_no.set(p - 1); (do_search)(); }
                                }
                            }
                        ) { "← 上一页" }
                        span(class="text-[#94a3b8] text-xs") { (cur + 1) " / " (total_pages) }
                        button(
                            class="px-3 py-1.5 rounded-lg border border-[#e4e7ef] text-[#64748b] hover:bg-[#f8fafc] disabled:opacity-40 text-xs font-medium",
                            disabled=cur + 1 >= total_pages,
                            on:click={
                                let do_search = do_search.clone();
                                move |_| {
                                    let next_page = page_no.get_clone() + 1;
                                    page_no.set(next_page);
                                    (do_search)();
                                }
                            }
                        ) { "下一页 →" }
                    }
                }
            } else { view! {} })

            (if detail_open.get() {
                view! {
                    div(
                        class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4",
                        on:click=move |_| detail_open.set(false)
                    ) {
                        div(
                            class="card w-full max-w-3xl max-h-[80vh] p-0 flex flex-col overflow-hidden",
                            on:click=move |e: web_sys::MouseEvent| e.stop_propagation()
                        ) {
                            div(class="px-4 py-3 border-b border-[#e4e7ef] flex items-center justify-between") {
                                div(class="text-sm font-semibold text-[#1a2236] truncate") { (detail_title.get_clone()) }
                                button(
                                    class="px-2 py-1 text-xs rounded border border-[#e4e7ef] text-[#64748b] hover:bg-[#f8fafc]",
                                    on:click=move |_| detail_open.set(false)
                                ) { "关闭" }
                            }
                            div(class="p-4 overflow-auto text-sm text-[#1a2236] leading-relaxed") {
                                (if detail_loading.get() {
                                    view! { "加载中…" }
                                } else if let Some(e) = detail_error.get_clone() {
                                    view! { ("加载详情失败: ".to_string() + &e) }
                                } else if let Some(html) = detail_html.get_clone() {
                                    view! { div(class="md-render", dangerously_set_inner_html=html) }
                                } else {
                                    view! { "暂无内容" }
                                })
                            }
                        }
                    }
                }
            } else { view! {} })
        }
    }
}
