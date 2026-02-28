pub fn format_stats_table(total_time_ms: u64, stats: &serde_json::Value) -> String {
    let fmt_num = |v: Option<&serde_json::Value>| -> String {
        match v.and_then(|v| v.as_u64()) {
            Some(n) => {
                let s = n.to_string();
                let mut result = String::new();
                for (i, c) in s.chars().rev().enumerate() {
                    if i > 0 && i % 3 == 0 { result.push('_'); }
                    result.push(c);
                }
                result.chars().rev().collect()
            }
            None => "-".to_string(),
        }
    };

    let fmt_u64 = |n: u64| -> String {
        let s = n.to_string();
        let mut result = String::new();
        for (i, c) in s.chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 { result.push('_'); }
            result.push(c);
        }
        result.chars().rev().collect()
    };

    let w = (22, 10, 10, 8);
    let top = format!("┌{:─<w0$}┬{:─<w1$}┬{:─<w2$}┬{:─<w3$}┐", "", "", "", "",
                      w0 = w.0 + 2, w1 = w.1 + 2, w2 = w.2 + 2, w3 = w.3 + 2);
    let bot = format!("└{:─<w0$}┴{:─<w1$}┴{:─<w2$}┴{:─<w3$}┘", "", "", "", "",
                      w0 = w.0 + 2, w1 = w.1 + 2, w2 = w.2 + 2, w3 = w.3 + 2);

    let row = |name: &str, time: String, tokens: String, calls: String| {
        format!("│ {:<w0$} │ {:>w1$} │ {:>w2$} │ {:>w3$} │",
                name, time, tokens, calls,
                w0 = w.0, w1 = w.1, w2 = w.2, w3 = w.3)
    };

    let mut lines = vec![
        format!("Stream completed in {}ms", fmt_u64(total_time_ms)),
        top,
        row("Name", "Time (ms)".into(), "Tokens".into(), "Calls".into()),
        row("─".repeat(w.0).as_str(), "─".repeat(w.1).into(), "─".repeat(w.2).into(), "─".repeat(w.3).into()),
    ];

    if let Some(obj) = stats.as_object() {
        lines.push(row(
            "orchestrator",
            fmt_num(obj.get("orchestrator_time")),
            fmt_num(obj.get("orchestrator_tokens")),
            fmt_num(obj.get("orchestrator_call")),
        ));
        lines.push(row(
            "router",
            fmt_num(obj.get("router_time")),
            fmt_num(obj.get("router_tokens")),
            "-".into(),
        ));

        if let Some(workers) = obj.get("workers").and_then(|w| w.as_array()) {
            for worker in workers {
                let name = worker.get("worker_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                lines.push(row(
                    name,
                    fmt_num(worker.get("execution_time_ms")),
                    fmt_num(worker.get("tokens_used")),
                    fmt_num(worker.get("llm_calls")),
                ));
            }
        }
    }

    lines.push(bot);
    lines.join("\n")
}