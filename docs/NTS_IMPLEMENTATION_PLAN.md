# Plan d'implémentation NTS pour RKIK

## Vue d'ensemble
Ce document décrit l'implémentation complète de Network Time Security (NTS) dans RKIK en utilisant la bibliothèque `rkik-nts`.

## Contexte

### Qu'est-ce que NTS ?
Network Time Security (RFC 8915) est un mécanisme de sécurité cryptographique pour NTP qui utilise:
- **NTS-KE (Key Exchange)**: Authentification initiale et établissement de clés via TLS
- **NTS Extension Fields**: Chiffrement et authentification pendant la synchronisation NTP via des cookies opaques

### Pourquoi NTS ?
- **Authentification**: Garantit que les réponses temporelles proviennent du serveur légitime
- **Intégrité**: Empêche la manipulation des paquets NTP en transit
- **Anti-replay**: Protection contre les attaques par rejeu
- **Confidentialité partielle**: Les cookies sont chiffrés

## Architecture d'implémentation

### 1. Dépendances
```toml
[dependencies]
rkik-nts = "0.1"  # ou version Git si non publié sur crates.io
```

### 2. Structure des modules

```
src/
├── adapters/
│   ├── mod.rs
│   ├── ntp_client.rs        # Existant (rsntp)
│   ├── nts_client.rs        # NOUVEAU - Client NTS via rkik-nts
│   └── resolver.rs           # Existant
├── domain/
│   └── ntp.rs               # Modifier pour ajouter champ 'authenticated'
├── services/
│   ├── query.rs             # Modifier pour supporter NTS
│   └── compare.rs           # Modifier pour supporter NTS
└── bin/
    └── rkik.rs              # Ajouter options CLI NTS
```

### 3. Modifications du modèle de domaine

**src/domain/ntp.rs**
```rust
pub struct ProbeResult {
    pub target: Target,
    pub offset_ms: f64,
    pub rtt_ms: f64,
    pub stratum: u8,
    pub ref_id: String,
    pub utc: DateTime<Utc>,
    pub local: DateTime<Local>,
    pub timestamp: i64,
    pub authenticated: bool,  // NOUVEAU - indique si NTS était utilisé
}
```

### 4. Nouvel adaptateur NTS

**src/adapters/nts_client.rs**
```rust
use rkik_nts::{NtsClient, NtsClientConfig};
use std::time::Duration;
use crate::error::RkikError;

pub struct NtsTimeResult {
    pub network_time: DateTime<Utc>,
    pub offset_ms: f64,
    pub rtt_ms: f64,
    pub authenticated: bool,
    // Autres métadonnées NTS
}

pub async fn query_nts(
    server: &str,
    port: Option<u16>,
    timeout: Duration,
) -> Result<NtsTimeResult, RkikError> {
    // Configuration NTS
    let mut config = NtsClientConfig::new(server);
    if let Some(p) = port {
        config = config.with_port(p);
    }
    config = config.with_timeout(timeout);

    // Connexion et requête
    let mut client = NtsClient::new(config);
    client.connect().await?;
    let time = client.get_time().await?;

    // Conversion vers notre format
    Ok(NtsTimeResult {
        network_time: time.network_time,
        offset_ms: time.offset_signed(),
        rtt_ms: time.round_trip_delay_ms(),
        authenticated: time.authenticated,
    })
}
```

### 5. Options CLI

**Nouvelles options dans src/bin/rkik.rs**
```rust
#[derive(Parser, Debug)]
struct Args {
    // ... options existantes ...

    /// Enable NTS (Network Time Security) mode
    #[arg(long)]
    nts: bool,

    /// NTS-KE port (default: 4460)
    #[arg(long, default_value_t = 4460)]
    nts_port: u16,

    /// NTS timeout for key exchange in seconds
    #[arg(long, default_value_t = 10.0)]
    nts_timeout: f64,
}
```

### 6. Service de requête unifié

**Modification de src/services/query.rs**
```rust
pub async fn query_one(
    target: &str,
    ipv6: bool,
    timeout: Duration,
    use_nts: bool,        // NOUVEAU paramètre
    nts_port: u16,        // NOUVEAU paramètre
) -> Result<ProbeResult, RkikError> {
    if use_nts {
        // Branche NTS
        let nts_result = nts_client::query_nts(target, Some(nts_port), timeout).await?;
        // Convertir NtsTimeResult en ProbeResult
        Ok(ProbeResult {
            target: Target { ... },
            offset_ms: nts_result.offset_ms,
            rtt_ms: nts_result.rtt_ms,
            authenticated: true,  // NTS = toujours authentifié
            // ... autres champs
        })
    } else {
        // Branche NTP classique (code existant)
        let parsed = parse_target(target)?;
        // ... code existant ...
        Ok(ProbeResult {
            // ... champs existants ...
            authenticated: false,  // NTP classique = non authentifié
        })
    }
}
```

### 7. Formatage de la sortie

**Modifications des formatters**
- **Text**: Ajouter un indicateur visuel `[NTS ✓]` ou `[Authenticated]`
- **JSON**: Ajouter le champ `"authenticated": true/false`
- **Verbose**: Afficher des détails sur le NTS-KE (algorithmes, cookies, etc.)

### 8. Gestion des erreurs

Nouveaux types d'erreurs spécifiques à NTS:
- `NtsKeyExchangeFailed`: Échec du NTS-KE
- `NtsCertificateError`: Problème de certificat TLS
- `NtsNotSupported`: Serveur ne supporte pas NTS

## Serveurs NTS publics pour tests

- `time.cloudflare.com:4460`
- `nts.ntp.se:4460`
- `ntppool1.time.nl:4460`
- `time.txryan.com:123`
- `nts.ntp.org.au:4460`

## Exemples d'utilisation

```bash
# Requête NTS simple
rkik time.cloudflare.com --nts

# Requête NTS avec verbose
rkik time.cloudflare.com --nts -v

# Comparaison NTS vs NTP classique
rkik --compare time.cloudflare.com time.google.com --nts

# Mode JSON avec NTS
rkik nts.ntp.se --nts -j -p

# Mode infini avec NTS
rkik time.cloudflare.com --nts -8 -i 5.0
```

## Plan d'exécution

1. ✅ Recherche et compréhension (FAIT)
2. ✅ Création de la branche (FAIT)
3. ⏳ Ajout de la dépendance rkik-nts
4. ⏳ Création de l'adaptateur NTS
5. ⏳ Mise à jour du modèle de domaine
6. ⏳ Ajout des options CLI
7. ⏳ Modification du service de requête
8. ⏳ Mise à jour des formatters
9. ⏳ Tests d'intégration
10. ⏳ Documentation

## Considérations techniques

### Compatibilité
- NTS nécessite Tokio (déjà présent ✓)
- GPL-2.0 license pour rkik-nts (vérifier compatibilité avec MIT)

### Performance
- NTS-KE ajoute une latence initiale (échange TLS)
- Cookies peuvent être réutilisés pour plusieurs requêtes
- Considérer un cache de session NTS pour mode infini

### Sécurité
- Valider les certificats TLS par défaut
- Option `--nts-insecure` pour skip validation (dev only)
- Timeout approprié pour NTS-KE (plus long que NTP)

## Tests

### Tests unitaires
- Parser les options NTS CLI
- Conversion NtsTimeResult → ProbeResult
- Gestion d'erreurs NTS

### Tests d'intégration
- Requête NTS réussie sur serveurs publics
- Gestion de serveur NTS invalide
- Comparaison NTS vs NTP
- Mode infini avec NTS

## Documentation à mettre à jour

1. **README.md**: Ajouter section NTS
2. **docs/**: Créer guide NTS détaillé
3. **--help**: Documenter nouvelles options
4. **Examples**: Ajouter exemples NTS

---

**Sources**:
- [rkik-nts GitHub](https://github.com/aguacero7/rkik-nts)
- [RFC 8915](https://datatracker.ietf.org/doc/html/rfc8915)
- [Internet Society NTS Blog](https://www.internetsociety.org/blog/2020/10/nts-rfc-published-new-standard-to-ensure-secure-time-on-the-internet/)
