## Response

```Rust
pub enum Response<A, S> {
    Success(A, S),
    Reject
}
```

Une réponse est un type générique pour lequel:
- `A` est la valeur associée au résultat et 
- `S` est l'information associée à la source.

On distingue alors deux cas: 
- un succès et 
- un échec.

## Alternative 

Le type Rust `Result` peut tout aussi bien remplacé 
cet enumération comme suit:

```Rust
type Response<A,S> = Result<(A, S),()>;
```

Par contre cela necessite une réécriture du fold.
