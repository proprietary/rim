use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub fn toposort_files(files: &Vec<PathBuf>) -> Vec<PathBuf> {
    let mut graph: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    for file in files {
        let path = file.clone();
        for ancestor in path.ancestors().skip(1) {
            if ancestor.has_root() && ancestor.components().count() == 1 {
                continue;
            }
            let child_name = path.clone();
            graph
                .entry(ancestor.to_path_buf())
                .or_default()
                .push(child_name);
        }
    }
    topological_sort(&graph)
}

fn topological_sort(graph: &HashMap<PathBuf, Vec<PathBuf>>) -> Vec<PathBuf> {
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut sorted_paths: Vec<PathBuf> = Vec::new();

    fn dfs(
        node: &PathBuf,
        graph: &HashMap<PathBuf, Vec<PathBuf>>,
        visited: &mut HashSet<PathBuf>,
        sorted_paths: &mut Vec<PathBuf>,
    ) {
        if visited.contains(node) {
            return;
        }
        visited.insert(node.to_path_buf());
        for child in graph.get(node).unwrap_or(&Vec::new()) {
            dfs(child, graph, visited, sorted_paths);
        }
        sorted_paths.push(node.to_path_buf());
    }

    for node in graph.keys() {
        dfs(node, graph, &mut visited, &mut sorted_paths);
    }
    sorted_paths
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_toposort_files() {
        let mut files = vec![
            PathBuf::from("/tmp"),
            PathBuf::from("/tmp/foo/bar/baz/quux"),
            PathBuf::from("/tmp/foo"),
            PathBuf::from("/tmp/foo/bar/baz/qux"),
            PathBuf::from("/tmp/foo/bar/baz"),
            PathBuf::from("/tmp/foo/bar"),
        ];
        let sorted = toposort_files(&files);
        let expected = vec![
            PathBuf::from("/tmp/foo/bar/baz/quux"),
            PathBuf::from("/tmp/foo/bar/baz/qux"),
            PathBuf::from("/tmp/foo/bar/baz"),
            PathBuf::from("/tmp/foo/bar"),
            PathBuf::from("/tmp/foo"),
            PathBuf::from("/tmp"),
        ];
        assert_eq!(sorted, expected);
    }
}
