// Testes unitários para FileScanner

use rustdrivesync::sync::FileScanner;
use tempfile::TempDir;
use tokio::fs;

async fn create_test_files(dir: &TempDir) -> std::io::Result<()> {
    // Criar estrutura de diretórios
    let sub1 = dir.path().join("subdir1");
    let sub2 = dir.path().join("subdir2");
    let nested = sub1.join("nested");

    fs::create_dir_all(&sub1).await?;
    fs::create_dir_all(&sub2).await?;
    fs::create_dir_all(&nested).await?;

    // Criar arquivos de teste
    fs::write(dir.path().join("file1.txt"), b"content 1").await?;
    fs::write(dir.path().join("file2.txt"), b"content 2 larger").await?;
    fs::write(dir.path().join(".hidden"), b"hidden file").await?;

    fs::write(sub1.join("file3.txt"), b"content 3").await?;
    fs::write(sub1.join("file4.log"), b"log file content").await?;

    fs::write(nested.join("file5.txt"), b"deeply nested").await?;

    fs::write(sub2.join("file6.txt"), b"another file").await?;
    fs::write(sub2.join("file7.md"), b"# Markdown").await?;

    Ok(())
}

#[tokio::test]
async fn test_scanner_basic() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_files(&temp_dir).await.unwrap();

    let scanner = FileScanner::new();
    let files = scanner.scan_directory(temp_dir.path()).unwrap();

    // Deve encontrar todos os arquivos (incluindo .hidden se não ignorar)
    assert!(!files.is_empty());
}

#[tokio::test]
async fn test_scanner_recursive() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_files(&temp_dir).await.unwrap();

    let scanner = FileScanner::new();
    let files = scanner.scan_directory(temp_dir.path()).unwrap();

    // Deve encontrar arquivos em subdiretórios
    let paths: Vec<_> = files.iter().map(|f| f.relative_path.to_str().unwrap()).collect();

    // Verificar que encontrou arquivo em nested
    assert!(paths.iter().any(|p| p.contains("nested")));
}

#[tokio::test]
async fn test_scanner_with_ignore_hidden() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_files(&temp_dir).await.unwrap();

    let scanner = FileScanner::new()
        .with_ignore_patterns(vec![".hidden".to_string()]); // Ignora arquivo .hidden especificamente
    let files = scanner.scan_directory(temp_dir.path()).unwrap();

    // Não deve encontrar .hidden
    let names: Vec<_> = files
        .iter()
        .map(|f| f.relative_path.file_name().unwrap().to_str().unwrap())
        .collect();

    assert!(!names.contains(&".hidden"));
}

#[tokio::test]
async fn test_scanner_with_max_file_size() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_files(&temp_dir).await.unwrap();

    // Apenas arquivos com até 10 bytes
    let scanner = FileScanner::new().with_max_file_size(10);
    let files = scanner.scan_directory(temp_dir.path()).unwrap();

    // Todos os arquivos devem ter <= 10 bytes
    for file in &files {
        assert!(file.size <= 10);
    }
}

#[tokio::test]
async fn test_scanner_with_ignore_patterns() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_files(&temp_dir).await.unwrap();

    let scanner = FileScanner::new()
        .with_ignore_patterns(vec![".log".to_string(), ".md".to_string()]);
    let files = scanner.scan_directory(temp_dir.path()).unwrap();

    let names: Vec<_> = files
        .iter()
        .map(|f| f.relative_path.file_name().unwrap().to_str().unwrap())
        .collect();

    // Não deve encontrar .log nem .md (contains verifica se o caminho contém essas strings)
    assert!(!names.iter().any(|n| n.ends_with(".log")));
    assert!(!names.iter().any(|n| n.ends_with(".md")));
}

#[tokio::test]
async fn test_scanner_with_directory_pattern() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_files(&temp_dir).await.unwrap();

    let scanner = FileScanner::new()
        .with_ignore_patterns(vec!["subdir1".to_string()]);
    let files = scanner.scan_directory(temp_dir.path()).unwrap();

    let paths: Vec<_> = files.iter().map(|f| f.relative_path.to_str().unwrap()).collect();

    // Não deve encontrar arquivos em subdir1
    assert!(!paths.iter().any(|p| p.contains("subdir1")));
    // Mas deve encontrar em subdir2
    assert!(paths.iter().any(|p| p.contains("subdir2")));
}

#[tokio::test]
async fn test_scanner_empty_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    // Não criar arquivos

    let scanner = FileScanner::new();
    let files = scanner.scan_directory(temp_dir.path()).unwrap();

    assert_eq!(files.len(), 0);
}

#[tokio::test]
async fn test_scanner_nonexistent_directory() {
    let scanner = FileScanner::new();
    let result = scanner.scan_directory("/path/that/does/not/exist");

    assert!(result.is_err());
}

#[tokio::test]
async fn test_scanner_file_metadata() {
    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    let test_content = b"Test content with specific size";
    fs::write(&test_file, test_content).await.unwrap();

    let scanner = FileScanner::new();
    let files = scanner.scan_directory(temp_dir.path()).unwrap();

    assert_eq!(files.len(), 1);

    let file_info = &files[0];
    assert_eq!(file_info.size, test_content.len() as u64);
    assert_eq!(
        file_info.relative_path.file_name().unwrap(),
        "test.txt"
    );
    assert!(file_info.modified > 0);
}

#[tokio::test]
async fn test_scanner_with_symlinks() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(temp_dir.path().join("real.txt"), b"real file").await.unwrap();

    // Criar symlink (pode falhar no Windows)
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let link_path = temp_dir.path().join("link.txt");
        let _ = symlink(temp_dir.path().join("real.txt"), &link_path);
    }

    let scanner = FileScanner::new();
    let files = scanner.scan_directory(temp_dir.path()).unwrap();

    // Deve encontrar pelo menos o arquivo real
    assert!(!files.is_empty());
}

#[tokio::test]
async fn test_scanner_multiple_ignore_patterns() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_files(&temp_dir).await.unwrap();

    let scanner = FileScanner::new().with_ignore_patterns(vec![
        ".log".to_string(),
        ".md".to_string(),
        "subdir1".to_string(),
        ".hidden".to_string(),
    ]);
    let files = scanner.scan_directory(temp_dir.path()).unwrap();

    let paths: Vec<_> = files.iter().map(|f| f.relative_path.to_str().unwrap()).collect();

    // Não deve encontrar nada de subdir1, .log, .md ou .hidden (contains)
    assert!(!paths.iter().any(|p| p.contains("subdir1")));
    assert!(!paths.iter().any(|p| p.contains(".log")));
    assert!(!paths.iter().any(|p| p.contains(".md")));
    assert!(!paths.iter().any(|p| p.contains(".hidden")));

    // Mas deve encontrar arquivos de subdir2
    assert!(paths.iter().any(|p| p.contains("subdir2")));
}

#[tokio::test]
async fn test_scanner_zero_max_size() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_files(&temp_dir).await.unwrap();

    let scanner = FileScanner::new().with_max_file_size(0);
    let files = scanner.scan_directory(temp_dir.path()).unwrap();

    // Com max_size = 0, nenhum arquivo deve ser encontrado
    assert_eq!(files.len(), 0);
}

#[tokio::test]
async fn test_scanner_large_max_size() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_files(&temp_dir).await.unwrap();

    // Max size muito grande (1GB)
    let scanner = FileScanner::new().with_max_file_size(1_000_000_000);
    let files = scanner.scan_directory(temp_dir.path()).unwrap();

    // Deve encontrar todos os arquivos
    assert!(!files.is_empty());
}

#[tokio::test]
async fn test_scanner_chaining_configuration() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_files(&temp_dir).await.unwrap();

    let scanner = FileScanner::new()
        .with_max_file_size(50)
        .with_ignore_patterns(vec![".hidden".to_string(), ".log".to_string()]);

    let files = scanner.scan_directory(temp_dir.path()).unwrap();

    // Todos os filtros devem ser aplicados
    let names: Vec<_> = files
        .iter()
        .map(|f| f.relative_path.file_name().unwrap().to_str().unwrap())
        .collect();

    assert!(!names.contains(&".hidden")); // ignorar .hidden
    assert!(!names.iter().any(|n| n.contains(".log"))); // ignorar .log (contains)

    // Verificar tamanho máximo
    for file in &files {
        assert!(file.size <= 50);
    }
}
