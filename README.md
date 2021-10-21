## Programmation par Traits sous l'influence de la gestion mémoire

[![Build Status](https://travis-ci.org/d-plaindoux/rust-traits.svg?branch=master)](https://travis-ci.org/d-plaindoux/rust-traits)


On nous parle trop souvent de Rust en terme de gestion mémoire avec le borrowing et le lifetime mais quid de la conception logiciel dans un tel contexte ?

Je propose d'explorer le langage en ayant une approche objet très naive pour ensuite dériver et voir comment Rust nous propose des chemins différents. L'ouverture du code - pour une plus grande réutilisabilité et adaptabilité - dirigée par la gestion mémoire est une nouveauté et va nous permettre de pousser l'abstraction au maximum en utilisant les génériques et les contraintes de typage pour notre plus grand bien ! Nous finirons par explorer l'"expression problem" de Wadler et voir comment Rust peut y répondre.

Présentations disponibles:
- [BreizhCamp 2019](https://www.youtube.com/watch?v=szrR4Klixdk)
- [LilleFP 2020](https://www.youtube.com/watch?v=6CO98XBsNiY)
