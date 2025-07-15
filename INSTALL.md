# Guide d'installation - Mapscow Mule

## Installation de Rust

Mapscow Mule est développé en Rust et nécessite l'installation de Rust et Cargo.

### Méthode 1 : Installation officielle (Recommandée)

1. Allez sur https://rustup.rs/
2. Téléchargez et exécutez `rustup-init.exe`
3. Suivez les instructions (acceptez les paramètres par défaut)
4. Redémarrez votre terminal/PowerShell

### Méthode 2 : Via Chocolatey (si installé)

```powershell
choco install rust
```

### Méthode 3 : Via Scoop (si installé)

```powershell
scoop install rust
```

## Vérification de l'installation

Ouvrez un nouveau terminal PowerShell et tapez :

```powershell
rustc --version
cargo --version
```

Vous devriez voir quelque chose comme :
```
rustc 1.75.0 (82e1608df 2023-12-21)
cargo 1.75.0 (1d8b05cdd 2023-11-20)
```

## Compilation de Mapscow Mule

Une fois Rust installé :

```powershell
cd "c:\Users\yann\Documents\Yann\Code\mapscow-mule"
cargo build
```

## Exécution

```powershell
cargo run
```

## Dépannage

### Erreur "cargo: The term 'cargo' is not recognized"
- Redémarrez votre terminal après installation de Rust
- Vérifiez que le PATH inclut `%USERPROFILE%\.cargo\bin`

### Erreurs de compilation
- Assurez-vous d'avoir Rust 1.70 ou plus récent
- Mettez à jour avec `rustup update`

### Problèmes de dépendances
- Nettoyez le cache : `cargo clean`
- Rebuilder : `cargo build`
