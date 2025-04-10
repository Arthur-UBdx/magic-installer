# spec interface

## SP-UI-001

L'utilisateur doit pouvoir choisir entre 3 actions:
    - Installer/mettre à jour le modpack
    - Installer le modloader
    - Supprimer les mods
    - Ouvrir le fichier de configuration
    - Quitter

## SP-FN-001

L'utilitaire doit lire un fichier de configuration pour obtenir les informations
nécessaires à son fonctionnement. Sinon il doit utiliser une configuration par défaut.

## SP-FN-002

Lors du choix de l'action "Installer/mettre à jour le modpack", l'utilitaire doit

1. Télécharger le modpack depuis un serveur distant dont l'adresse est spécifiée dans le
    fichier de configuration tout en rendant compte à l'utilisateur à travers l'interface.
2. Extraire le contenu du modpack dans le dossier de jeu Minecraft puis remplacer/fusionner
    les dossiers et fichiers selon la directive specifiée dans la configuration.

## SP-FN-003

Lors du choix de l'action "Installer le modloader", l'utilitaire doit

1. Télécharger le modloader depuis un serveur distant dont l'adresse est spécifiée dans le
    fichier de configuration tout en rendant compte à l'utilisateur à travers l'interface.
2. Extraire le contenu du modloader
3. Lancer l'éxecutable d'installation du modloader

## SP-FN-004

Lors du choix de l'action "Supprimer les mods", l'utilitaire doit
supprimer les dossiers spécifiés dans la configuration.

## SP-PT-001

L'utilitaire doit être capable de fonctionner sur un système d'exploitation Windows
et Linux.

## SP-CF-001

La configuration doit contenir les informations suivantes:
    - Lien HTTP vers le serveur de téléchargement du modpack
    - Lien HTTP vers le serveur de téléchargement du modloader
    - Dossier de mods à remplacer si existant sinon créer / supprimer
    - Dossier de mods à fusionner si existant

## SP-CF-002

Le dossier de jeu minecraft doit pouvoir être spécifié par l'utilisateur.

## SP-FN-006

Le programme doit enregistrer des logs dans un fichier `logs.txt` dans `.minecraft/magic_installer`.
Toutes les erreurs entraînant la fin de l'éxecution du programme doivent être loggés.
