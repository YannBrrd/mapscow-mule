# Mapscow Mule - A Maperitive Clone in Rust

Un clone moderne de Maperitive développé en Rust, offrant un rendu de cartes haute qualité avec export SVG et une interface de personnalisation avancée.

## 🚀 Fonctionnalités

- **Rendu haute qualité** : Export SVG vectoriel de qualité professionnelle
- **Support OpenStreetMap** : Import et rendu des données OSM
- **Interface moderne** : Interface utilisateur native avec egui
- **Personnalisation avancée** : Éditeur de styles intégré
- **Support GPX** : Import et affichage des traces GPX
- **Multi-format** : Export SVG, PNG, JPEG
- **Performance** : Rendu optimisé en Rust

## 🏗️ Architecture

```
src/
├── main.rs              # Point d'entrée de l'application
├── app.rs               # Application principale et logique GUI
├── core/                # Types de données et géométrie de base
│   ├── mod.rs
│   ├── geometry.rs      # Utilitaires géométriques
│   └── projection.rs    # Systèmes de projection
├── parsers/             # Parseurs pour différents formats
│   ├── mod.rs
│   ├── osm.rs          # Parser OSM XML
│   ├── gpx.rs          # Parser GPX
│   └── stylesheet.rs    # Parser de feuilles de style
├── rendering/           # Moteur de rendu
│   └── mod.rs
├── styles/              # Gestionnaire de styles
│   └── mod.rs
├── export/              # Modules d'export
│   ├── mod.rs
│   ├── svg_export.rs   # Export SVG haute qualité
│   └── png_export.rs   # Export raster
├── gui/                 # Interface utilisateur
│   ├── mod.rs
│   ├── map_view.rs     # Vue carte principale
│   ├── style_editor.rs # Éditeur de styles
│   ├── tool_panel.rs   # Panneau d'outils
│   └── widgets.rs      # Widgets personnalisés
└── utils/               # Utilitaires
    ├── mod.rs
    ├── file_dialog.rs
    └── config.rs
```

## 🛠️ Installation

### Prérequis
- Rust 1.70+ 
- Windows (support Linux/macOS à venir)

### Compilation
```bash
git clone https://github.com/YannBrrd/mapscow-mule.git
cd mapscow-mule
cargo build --release
```

### Lancement
```bash
cargo run
```

## 📖 Utilisation

### Interface graphique
1. **Fichier → Ouvrir fichier OSM** : Charger des données OpenStreetMap
2. **Fichier → Ouvrir fichier GPX** : Ajouter des traces GPX
3. **Vue → Éditeur de styles** : Personnaliser l'apparence de la carte
4. **Fichier → Exporter** : Générer des fichiers SVG/PNG

### Mode ligne de commande
```bash
mapscow-mule --headless --config my_config.yaml
```

## 🎨 Personnalisation des styles

Le système de styles supporte :
- Sélecteurs basés sur les tags OSM
- Couleurs, épaisseurs de ligne, remplissages
- Texte et étiquettes personnalisables
- Plages de zoom
- Format YAML ou syntaxe Maperitive

### Exemple de style
```yaml
rules:
  - selectors:
      - tag: { key: "highway", value: "primary" }
    style:
      draw_mode: Line
      line_color: { r: 255, g: 200, b: 100, a: 255 }
      line_width: 3.0
```

## 🚧 État du développement

- [x] Architecture de base
- [x] Parser OSM
- [x] Parser GPX  
- [x] Interface utilisateur de base
- [x] Export SVG
- [x] Export PNG/JPEG
- [x] Éditeur de styles
- [ ] Système de cache
- [ ] Support des relations OSM
- [ ] Ombrage de terrain
- [ ] Mode batch/ligne de commande
- [ ] Support des tuiles

## 🤝 Contribution

Les contributions sont les bienvenues ! Voir [CONTRIBUTING.md](CONTRIBUTING.md) pour les détails.

## 📄 Licence

Ce projet est sous licence MIT. Voir [LICENSE](LICENSE) pour plus de détails.

## 🙏 Remerciements

Inspiré par [Maperitive](http://maperitive.net/) d'Igor Brejc, un excellent outil de cartographie.
