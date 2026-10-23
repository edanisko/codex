//! Tests for local-link parsing and root-preserving separator normalization.

use super::trim_trailing_local_path_separator;
use crate::markdown_render::preferences;
use crate::markdown_render::render_markdown_lines_with_width_and_cwd;
use codex_config::types::TuiRendering;
use pretty_assertions::assert_eq;
use std::path::Path;
use url::Url;

fn render_links(
    markdown: &str,
    workspace: &Path,
    width: usize,
) -> Vec<crate::terminal_hyperlinks::HyperlinkLine> {
    render_markdown_lines_with_width_and_cwd(markdown, Some(width), Some(workspace))
}

fn enable_links() {
    preferences::init(TuiRendering {
        local_file_links: true,
        ..Default::default()
    });
}

#[test]
fn workspace_links_are_opt_in_and_disabled_for_remote_workspaces() {
    let workspace = tempfile::tempdir().unwrap();
    std::fs::write(workspace.path().join("report.md"), "report").unwrap();
    let markdown = "[Read report](./report.md)";
    assert!(
        render_links(markdown, workspace.path(), 80)
            .iter()
            .all(|line| line.hyperlinks.is_empty())
    );
    enable_links();
    assert!(
        render_links(markdown, workspace.path(), 80)
            .iter()
            .any(|line| !line.hyperlinks.is_empty())
    );
    preferences::set_workspace_is_local(false);
    assert!(
        render_links(markdown, workspace.path(), 80)
            .iter()
            .all(|line| line.hyperlinks.is_empty())
    );
}

#[test]
fn document_labels_and_destinations_survive_wrapping_tables_and_streaming() {
    enable_links();
    let workspace = tempfile::tempdir().unwrap();
    let target = workspace.path().join("research résumé.zip");
    std::fs::write(&target, "archive placeholder").unwrap();
    let destination = Url::from_file_path(target.canonicalize().unwrap())
        .unwrap()
        .to_string();
    for markdown in [
        "[Read the complete research](<research résumé.zip>)".to_string(),
        format!("[Read the complete research]({destination})"),
        "| Download |\n|---|\n| [Read the complete research](<./research résumé.zip>) |"
            .to_string(),
    ] {
        for width in [14, 30, 80] {
            let completed = render_links(&markdown, workspace.path(), width);
            let links: Vec<_> = completed.iter().flat_map(|line| &line.hyperlinks).collect();
            assert!(!links.is_empty(), "{markdown} at width {width}");
            for link in links {
                assert_eq!(
                    link.terminal_destination().as_deref(),
                    Some(destination.as_str())
                );
                assert!(link.columns.start < link.columns.end);
            }
            for line in completed.iter().filter(|line| !line.hyperlinks.is_empty()) {
                let emitted: String = crate::terminal_hyperlinks::decorate_spans(line)
                    .iter()
                    .map(|span| span.content.as_ref())
                    .collect();
                assert!(emitted.contains(&format!("\x1b]8;;{destination}\x07")));
                assert!(emitted.contains("\x1b]8;;\x07"));
                assert_eq!(
                    crate::terminal_hyperlinks::strip_osc8(&emitted),
                    line.line.to_string(),
                );
            }
            let streamed =
                crate::markdown_render::render_streaming_markdown_lines_with_width_and_cwd(
                    &markdown,
                    Some(width),
                    Some(workspace.path()),
                    &|_| false,
                    crate::markdown_render::ListSpacing::AfterMultiline,
                );
            assert_eq!(streamed.lines, completed);
        }
    }
    let rendered = render_links("[Download](<research résumé.zip>)", workspace.path(), 80);
    assert_eq!(rendered[0].line.to_string(), "Download");
}

#[test]
fn local_document_resolution_handles_absolute_paths_spaces_and_locations() {
    enable_links();
    let workspace = tempfile::tempdir().unwrap();
    let target = workspace.path().join("report one.md");
    std::fs::write(&target, "report").unwrap();
    let base_url = Url::from_file_path(target.canonicalize().unwrap()).unwrap();
    for (destination, fragment) in [
        ("./report%20one.md:12:3".to_string(), "L12C3"),
        (format!("{}#L12C3", target.display()), "L12C3"),
        (format!("{base_url}#L12C3"), "L12C3"),
    ] {
        let file = super::resolve_workspace_document(&destination, workspace.path()).unwrap();
        let hyperlink =
            crate::terminal_hyperlinks::TerminalHyperlink::trusted_workspace_file(0..1, file);
        let url = Url::parse(&hyperlink.terminal_destination().unwrap()).unwrap();
        assert_eq!(url.to_file_path().unwrap(), target.canonicalize().unwrap());
        assert_eq!(url.fragment(), Some(fragment));
    }
}

#[test]
fn local_document_resolution_rejects_untrusted_targets() {
    let workspace = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let outside_file = outside.path().join("outside.md");
    std::fs::write(&outside_file, "outside").unwrap();
    std::fs::write(workspace.path().join(".hidden.md"), "hidden").unwrap();
    std::fs::write(workspace.path().join("run.command"), "executable").unwrap();
    for destination in [
        "missing.md",
        ".hidden.md",
        "run.command",
        "./",
        "file://remote-server/report.md",
        "https://example.com/report.md",
        "javascript:alert(1)",
        "./report%00.md",
        "./report%1b.md",
        "~/report.md",
        outside_file.to_str().unwrap(),
    ] {
        assert!(
            super::resolve_workspace_document(destination, workspace.path()).is_none(),
            "{destination}"
        );
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(outside_file, workspace.path().join("escape.md")).unwrap();
        std::os::unix::fs::symlink(
            workspace.path().join("run.command"),
            workspace.path().join("run.md"),
        )
        .unwrap();
        for destination in ["escape.md", "run.md"] {
            assert!(super::resolve_workspace_document(destination, workspace.path()).is_none());
        }
    }
}

#[test]
fn trailing_separator_trimming_preserves_local_roots() {
    let paths = ["/", "//", "C:/", "dir/", "//server/share/", "C:/dir/"];
    assert_eq!(
        paths.map(trim_trailing_local_path_separator),
        ["/", "//", "C:/", "dir", "//server/share", "C:/dir"]
    );
}
