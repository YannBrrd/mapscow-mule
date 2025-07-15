# Guide d'utilisation - Mapscow Mule

## Démarrage rapide

### 1. Installation
Suivez les instructions dans [INSTALL.md](INSTALL.md) pour installer Rust et compiler l'application.

### 2. Premier lancement
```bash
cargo run
```

L'application démarre avec l'interface graphique principale.

## Interface utilisateur

### Barre de menu
- **Fichier** : Ouvrir des fichiers OSM/GPX, exporter des cartes
- **Vue** : Afficher/masquer les panneaux
- **Aide** : À propos de l'application

### Panneaux

#### Panneau d'outils (gauche)
- **Navigation** : Outils de pan et zoom
- **Mesure** : Outils de mesure de distance
- **Couches** : Activer/désactiver les types de données
- **Export** : Raccourcis vers les exports

#### Vue carte (centre)
- **Navigation souris** : 
  - Clic + glisser : déplacer la carte
  - Molette : zoomer/dézoomer
- **Informations** : Coordonnées et niveau de zoom affichés

#### Éditeur de styles (droite)
- **Liste des règles** : Toutes les règles de style actives
- **Éditeur** : Modifier les propriétés de style sélectionnées

## Formats supportés

### Import
- **OSM XML** (`.osm`) : Données OpenStreetMap
- **GPX** (`.gpx`) : Traces GPS
- **Styles** (`.yaml`) : Feuilles de style personnalisées

### Export
- **SVG** (`.svg`) : Format vectoriel haute qualité
- **PNG** (`.png`) : Image raster
- **JPEG** (`.jpg`) : Image raster compressée

## Exemples d'utilisation

### 1. Charger une carte OSM

1. **Fichier → Ouvrir fichier OSM**
2. Sélectionnez `examples/notre-dame.osm`
3. La carte s'affiche avec le style par défaut

### 2. Ajouter une trace GPX

1. **Fichier → Ouvrir fichier GPX**
2. Sélectionnez `examples/notre-dame-walk.gpx`
3. La trace apparaît sur la carte

### 3. Personnaliser le style

1. Ouvrez l'**Éditeur de styles** (panneau droit)
2. Sélectionnez une règle existante ou créez-en une nouvelle
3. Modifiez les couleurs, épaisseurs, etc.
4. Les changements s'appliquent immédiatement

### 4. Exporter en SVG

1. **Fichier → Exporter comme SVG**
2. Choisissez le nom et l'emplacement
3. Configurez la résolution et les options
4. Cliquez **Exporter**

## Conseils d'utilisation

### Performance
- Les gros fichiers OSM peuvent prendre du temps à charger
- Utilisez le zoom pour naviguer efficacement
- Les exports haute résolution sont plus lents

### Styles
- Commencez avec le style par défaut
- Les règles sont appliquées dans l'ordre (première correspondance)
- Utilisez les tags OSM pour les sélecteurs

### Qualité d'export
- SVG : Qualité vectorielle parfaite, idéal pour l'impression
- PNG : Bon pour le web, configurez la résolution selon l'usage
- JPEG : Plus petit fichier, avec compression

## Raccourcis clavier

- **Ctrl+O** : Ouvrir fichier OSM
- **Ctrl+E** : Exporter
- **Ctrl+Q** : Quitter
- **F1** : Afficher l'aide
- **F11** : Plein écran

## Résolution de problèmes

### La carte ne s'affiche pas
- Vérifiez que le fichier OSM est valide
- Essayez de zoomer pour ajuster la vue
- Consultez les messages d'erreur dans la console

### Export échoue
- Vérifiez l'espace disque disponible
- Réduisez la résolution pour les gros exports
- Assurez-vous d'avoir les permissions d'écriture

### Performance lente
- Fermez les autres applications
- Réduisez la complexité du style
- Utilisez des fichiers OSM plus petits pour tester

## Données d'exemple

Le dossier `examples/` contient :
- `notre-dame.osm` : Données OSM simplifiées autour de Notre-Dame
- `notre-dame-walk.gpx` : Trace de promenade
- `paris-tourism.yaml` : Style optimisé pour le tourisme

Ces fichiers sont parfaits pour découvrir les fonctionnalités de l'application.
