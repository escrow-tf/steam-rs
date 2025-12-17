use derive_more::Into;
use num_enum::{TryFromPrimitive, TryFromPrimitiveError};
use serde::Deserialize;
use serde::de::Visitor;
use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;
use std::{fmt::Formatter, num::ParseIntError};
use thiserror::Error;

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Copy, Clone)]
#[repr(u8)]
pub enum Universe {
    Invalid = 0,
    Public = 1,
    Beta = 2,
    Internal = 3,
    Dev = 4,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Copy, Clone)]
#[repr(u8)]
pub enum Type {
    Invalid = 0,
    Individual = 1,
    Multiseat = 2,
    GameServer = 3,
    AnonymousGameServer = 4,
    Pending = 5,
    ContentServer = 6,
    Clan = 7,
    Chat = 8,
    PeerToPeerSuperSeeder = 9,
    AnonymousUser = 10,
}

#[derive(Debug, Eq, PartialEq, TryFromPrimitive, Copy, Clone)]
#[repr(u16)]
pub enum Instance {
    All = 0,
    Desktop = 1,
    Console = 2,
    Web = 3,
}

const ID_MASK: u64 = 0xFFFFFFFF;
const INSTANCE_MASK: u64 = 0x000FFFFF;
const TYPE_MASK: u64 = 0x0000000F;

#[derive(Error, Debug)]
pub enum ParseSteamIDError {
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),

    #[error(transparent)]
    ConvertSteamIDError(#[from] ConvertSteamIDError),
}

#[derive(Error, Debug)]
pub enum ConvertSteamIDError {
    #[error("account id must not be 0")]
    AccountIDIsZero,

    #[error("id instance must be 'All' when type is 'Clan'")]
    NonAllInstanceInClanID,

    #[error(transparent)]
    UniverseOutOfRange(#[from] TryFromPrimitiveError<Universe>),

    #[error(transparent)]
    TypeOutOfRange(#[from] TryFromPrimitiveError<Type>),

    #[error(transparent)]
    InstanceOutOfRange(#[from] TryFromPrimitiveError<Instance>),
}

/// # SteamID
/// A valid 64-bit Steam ID. There are several invariants, see [ParseSteamIDError].
///
/// Parse from a string:
///
/// ```rs
/// let steam_id = "76561197960287930".parse::<SteamID>()?;
/// ```
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct SteamID {
    pub universe: Universe,
    pub id_type: Type,
    pub instance: Instance,
    pub account_id: u32,
}

impl From<SteamID> for String {
    fn from(val: SteamID) -> Self {
        let id: u64 = val.into();
        id.to_string()
    }
}

impl Display for SteamID {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(String::from(*self).as_str())
    }
}

impl FromStr for SteamID {
    type Err = ParseSteamIDError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id: u64 = s.parse()?;
        SteamID::try_from(id).map_err(ParseSteamIDError::ConvertSteamIDError)
    }
}

impl From<SteamID> for u64 {
    fn from(val: SteamID) -> u64 {
        let universe = (val.universe as u64) << 56;
        let id_type = (val.id_type as u64) << 52;
        let instance = (val.instance as u64) << 32;
        let account_id = val.account_id as u64;

        universe | id_type | instance | account_id
    }
}

impl TryFrom<u64> for SteamID {
    type Error = ConvertSteamIDError;
    fn try_from(id: u64) -> Result<Self, Self::Error> {
        let account_id = (id & ID_MASK) as u32;
        let instance = Instance::try_from(((id >> 32) & INSTANCE_MASK) as u16)?;
        let id_type = Type::try_from(((id >> 52) & TYPE_MASK) as u8)?;
        let universe = Universe::try_from((id >> 56) as u8)?;

        if account_id == 0 {
            return Err(ConvertSteamIDError::AccountIDIsZero);
        }

        if id_type == Type::Clan && instance != Instance::All {
            return Err(ConvertSteamIDError::NonAllInstanceInClanID);
        }

        Ok(SteamID {
            universe,
            id_type,
            instance,
            account_id,
        })
    }
}

struct SteamIDVisitor;

impl<'de> Visitor<'de> for SteamIDVisitor {
    type Value = SteamID;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid 64-bit steam id")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse::<SteamID>()
            .map_err(|err| E::custom(format!("error parsing SteamID from value: {}", err)))
    }
}

impl<'de> Deserialize<'de> for SteamID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(SteamIDVisitor)
    }
}

#[derive(Error, Debug)]
pub enum ConvertPlayerSteamIDError {
    #[error("the ID's Universe must be set as Public")]
    UniverseIsNotPublic,

    #[error("the ID's Type must be set as Individual")]
    TypeIsNotIndividual,
}

/// [PlayerSteamID] is like a [SteamID], but it has the following additional invariants:
/// - the [Universe] is always Public
/// - the [Type] is always Individual
///
/// [SteamID] to [PlayerSteamID] with [TryFrom] or [TryInto]:
///
/// ```rs
/// let steam_id = "76561197960287930".parse::<SteamID>()?;
/// let player_steam_id = PlayerSteamID::try_from(steam_id)?;
/// ```
///
/// Convert back to [SteamID] with [From] or [Into]:
///
/// ```rs
/// let steam_id = SteamID::from(player_steam_id.into());
/// ```
#[derive(PartialEq, Debug, Into)]
pub struct PlayerSteamID {
    steam_id: SteamID,
}

impl TryFrom<SteamID> for PlayerSteamID {
    type Error = ConvertPlayerSteamIDError;
    fn try_from(id: SteamID) -> Result<Self, Self::Error> {
        if id.universe != Universe::Public {
            Err(ConvertPlayerSteamIDError::UniverseIsNotPublic)
        } else if id.id_type != Type::Individual {
            Err(ConvertPlayerSteamIDError::TypeIsNotIndividual)
        } else {
            Ok(PlayerSteamID { steam_id: id })
        }
    }
}

struct PlayerSteamIDVisitor;

impl<'de> Visitor<'de> for PlayerSteamIDVisitor {
    type Value = PlayerSteamID;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid 64-bit public individual steam id")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let steam_id = v
            .parse::<SteamID>()
            .map_err(|err| E::custom(format!("error parsing SteamID from value: {}", err)))?;

        PlayerSteamID::try_from(steam_id).map_err(|err| {
            E::custom(format!(
                "error creating PlayerSteamID from SteamID: {}",
                err
            ))
        })
    }
}

impl<'de> Deserialize<'de> for PlayerSteamID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(PlayerSteamIDVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches::assert_matches;

    #[test]
    fn test_steamid_invariants() {
        // testing invariants:
        // - valid 64-bit integer
        // - valid universe, instance, type, and account id
        assert_matches!(
            "".parse::<SteamID>(),
            Err(ParseSteamIDError::ParseIntError(_))
        );
        assert_matches!(
            "abcd".parse::<SteamID>(),
            Err(ParseSteamIDError::ParseIntError(_))
        );
        assert_matches!("76561197960287930".parse::<SteamID>(), Ok(_));

        // since string parsing succeeds normally, we can just test From<u64> invariants
        let invalid_id = form_id(
            Universe::Public as u8,
            Type::Clan as u16,
            Instance::Web as u8,
            1,
        );
        assert_matches!(
            SteamID::try_from(invalid_id),
            Err(ConvertSteamIDError::NonAllInstanceInClanID)
        );

        let invalid_id = form_id(0xFF, Type::Individual as u16, Instance::Web as u8, 1);
        assert_matches!(
            SteamID::try_from(invalid_id),
            Err(ConvertSteamIDError::UniverseOutOfRange(_))
        );

        let invalid_id = form_id(Universe::Public as u8, 0xFFFF, Instance::Web as u8, 1);
        assert_matches!(
            SteamID::try_from(invalid_id),
            Err(ConvertSteamIDError::TypeOutOfRange(_))
        );

        let invalid_id = form_id(Universe::Public as u8, Type::Individual as u16, 0xFF, 1);
        assert_matches!(
            SteamID::try_from(invalid_id),
            Err(ConvertSteamIDError::InstanceOutOfRange(_))
        );

        let invalid_id = form_id(
            Universe::Public as u8,
            Type::Individual as u16,
            Instance::Web as u8,
            0,
        );
        assert_matches!(
            SteamID::try_from(invalid_id),
            Err(ConvertSteamIDError::AccountIDIsZero)
        );

        let valid_id = form_id(
            Universe::Public as u8,
            Type::Individual as u16,
            Instance::Web as u8,
            1,
        );
        assert_matches!(SteamID::try_from(valid_id), Ok(_));

        fn form_id(universe: u8, id_type: u16, instance: u8, account_id: u32) -> u64 {
            let universe = (universe as u64) << 56;
            let id_type = (id_type as u64) << 52;
            let instance = (instance as u64) << 32;
            let account_id = account_id as u64;

            universe | id_type | instance | account_id
        }
    }

    #[test]
    fn test_player_steamid_invariants() {
        // testing invariants:
        // - universe must be Public
        // - type must be Individual

        let steam_id = "76561197960287930".parse::<SteamID>().unwrap();
        assert_matches!(PlayerSteamID::try_from(steam_id), Ok(_));

        let player_steam_id = PlayerSteamID::try_from(steam_id).unwrap();
        let steam_id = SteamID::from(player_steam_id);
        assert_eq!(u64::from(steam_id), 76561197960287930);

        let invalid_id = form_id(
            Universe::Beta as u8,
            Type::Individual as u16,
            Instance::Web as u8,
            1,
        );
        assert_matches!(
            PlayerSteamID::try_from(invalid_id),
            Err(ConvertPlayerSteamIDError::UniverseIsNotPublic)
        );

        let invalid_id = form_id(
            Universe::Public as u8,
            Type::Chat as u16,
            Instance::Web as u8,
            1,
        );
        assert_matches!(
            PlayerSteamID::try_from(invalid_id),
            Err(ConvertPlayerSteamIDError::TypeIsNotIndividual)
        );

        fn form_id(universe: u8, id_type: u16, instance: u8, account_id: u32) -> SteamID {
            let universe = (universe as u64) << 56;
            let id_type = (id_type as u64) << 52;
            let instance = (instance as u64) << 32;
            let account_id = account_id as u64;

            (universe | id_type | instance | account_id)
                .try_into()
                .unwrap()
        }
    }
}
