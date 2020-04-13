#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet certificate with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, ensure, print};
use frame_support::traits::{Randomness};
use system::{ ensure_signed, ensure_root };
use sp_std::prelude::Vec;
use codec::{Codec, Decode, Encode};
use sp_core::{H256, sr25519};
use sp_io::crypto::sr25519_verify;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Entity<Hash> {
    id: Hash,
    status: u8,
}

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
	// Add other types and constants required to configure this pallet.
	type Randomness: Randomness<H256>;

	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This pallet's storage items.
decl_storage! {
	// It is important to update your storage name so that your pallet's
	// storage items are isolated from other pallets.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as CertificateModule {
		// certifybook accounts
		OfficalAccountsArray get(offical_account_by_index): map hasher(twox_64_concat) u8 => T::AccountId;
		OfficalAccountsCount get(offical_accounts_count): u8;
		OfficalAccountsIndex: map hasher(blake2_128_concat) T::AccountId => u8;

		// entities
		Entities get(entities): map hasher(blake2_128_concat) H256 => Entity<H256>;
		EntitiesArray get(entity_by_index): map hasher(twox_64_concat) u32 => H256;
		EntitiesCount get(entities_count): u32;
		EntitiesIndex: map hasher(blake2_128_concat) H256 => u32;

		// managers and issuers of entity
		EntityManagers get(fn entity_managers): map hasher(blake2_128_concat) H256 => Vec<T::AccountId>;
		EntityIssuers get(fn entity_issuers): map hasher(blake2_128_concat) H256 => Vec<T::AccountId>;

		// certificates
		CertificatesArray get(certificate_by_index): map hasher(twox_64_concat) u64 => Vec<u8>;
		CertificatesCount get(certificates_count): u64;
		CertificatesIndex: map hasher(blake2_128_concat) Vec<u8> => u64;
		NonceByIssuerId get(nonce_by_issuer_id): map hasher(blake2_128_concat) T::AccountId => u64;

		// certificate's entity id
		EntityOfCertificate get(fn entity_of_certificate): map hasher(blake2_128_concat) Vec<u8> => H256;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {
		EntityCreated(AccountId, H256),
		IssuerAdded,
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Value was None
		NoneValue,
		/// Value reached maximum and cannot be incremented further
		StorageOverflow,
	}
}

// The pallet's dispatchable functions.
decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing errors
		// this includes information about your errors in the node's metadata.
		// it is needed only if you are using errors in your pallet
		type Error = Error<T>;

		// Initializing events
		// this is needed only if you are using events in your pallet
		fn deposit_event() = default;

		pub fn add_offical_account(origin, account_id: T::AccountId) -> dispatch::DispatchResult { 
			let _sender = ensure_root(origin)?;
			ensure!(!<OfficalAccountsIndex<T>>::contains_key(&account_id), "The account is aready in offical account list.");

			let offical_accounts_count = Self::offical_accounts_count();
			let new_offical_accounts_count = offical_accounts_count.checked_add(1).ok_or("Overflow adding a new account to offical account list")?;
			<OfficalAccountsArray<T>>::insert(offical_accounts_count, &account_id);
			<OfficalAccountsCount>::put(new_offical_accounts_count);
			<OfficalAccountsIndex<T>>::insert(&account_id, offical_accounts_count);

			Ok(())
		}

		pub fn remove_offical_account(origin, account_id: T::AccountId) -> dispatch::DispatchResult {
			let _sender = ensure_root(origin)?;
			ensure!(<OfficalAccountsIndex<T>>::contains_key(&account_id), "The account is not in offical account list.");

			let offical_accounts_count = Self::offical_accounts_count();
			let new_offical_accounts_count = offical_accounts_count.checked_sub(1).ok_or("Underflow removing a account from offical account list")?;
			<OfficalAccountsArray<T>>::remove(offical_accounts_count);
			<OfficalAccountsCount>::put(new_offical_accounts_count);
			<OfficalAccountsIndex<T>>::remove(&account_id);

			Ok(())
		}

		pub fn create_entity(origin, creator: T::AccountId) -> dispatch::DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(<OfficalAccountsIndex<T>>::contains_key(&sender), "You do not have permission to do this opertion!");

			// create a new entity
			// let new_entity_id = T::Randomness::random_seed();
			let entities_count = Self::entities_count();
			let new_entity_id = T::Randomness::random(&entities_count.to_be_bytes());
			ensure!(!<Entities>::contains_key(new_entity_id), "Entity already exists");

			let new_entities_count = entities_count.checked_add(1).ok_or("Overflow adding a new entity to total supply")?;

			let new_entity = Entity {
				id: new_entity_id,
				status: 1,
			};
			<Entities>::insert(new_entity_id, new_entity);
			<EntitiesArray>::insert(entities_count, new_entity_id);
			<EntitiesCount>::put(new_entities_count);
			<EntitiesIndex>::insert(new_entity_id, entities_count);

			// creator will be manager of the entity
			let mut entity_managers = Self::entity_managers(new_entity_id);
			entity_managers.push(creator.clone());
			<EntityManagers<T>>::insert(new_entity_id, entity_managers);

			// creator will be issuer of the entity
			let mut entity_issuers = Self::entity_issuers(new_entity_id);
			entity_issuers.push(creator.clone());
			<EntityIssuers<T>>::insert(new_entity_id, entity_issuers);

			// event
			Self::deposit_event(RawEvent::EntityCreated(creator, new_entity_id));

			Ok(())
		}

		pub fn add_manager(origin, entity_id: H256, manager_id: T::AccountId) {
			let sender = ensure_signed(origin)?;
			ensure!(<OfficalAccountsIndex<T>>::contains_key(&sender), "You do not have permission to do this opertion!");

			// add to manager list if not exist
			let mut entity_managers = Self::entity_managers(entity_id);
			ensure!(!entity_managers.contains(&manager_id), "This account is already a manager of the entity");
			entity_managers.push(manager_id);
			<EntityManagers<T>>::insert(entity_id, entity_managers);
		}

		pub fn remove_manager(origin, entity_id: H256, manager_id: T::AccountId) {
			let sender = ensure_signed(origin)?;
			ensure!(<OfficalAccountsIndex<T>>::contains_key(&sender), "You do not have permission to do this opertion!");

			// remove from manager list if exist
			let mut entity_managers = Self::entity_managers(entity_id);
			ensure!(entity_managers.contains(&manager_id), "This account is not a manager of the entity");
			entity_managers.retain(|x| x == &manager_id);
			<EntityManagers<T>>::insert(entity_id, entity_managers);
		}

		pub fn add_issuer(origin, entity_id: H256, issuer_id: T::AccountId) {
			let sender = ensure_signed(origin)?;
			ensure!(<OfficalAccountsIndex<T>>::contains_key(&sender), "You do not have permission to do this opertion!");

			// add to issuer list if not exist
			let mut entity_issuers = Self::entity_issuers(entity_id);
			ensure!(!entity_issuers.contains(&issuer_id), "This account is already an issuer of the entity");
			entity_issuers.push(issuer_id);
			<EntityIssuers<T>>::insert(entity_id, entity_issuers);

			Self::deposit_event(RawEvent::IssuerAdded);
		}

		pub fn remove_issuer(origin, entity_id: H256, issuer_id: T::AccountId) {
			let sender = ensure_signed(origin)?;
			ensure!(<OfficalAccountsIndex<T>>::contains_key(&sender), "You do not have permission to do this opertion!");

			// remove from issuer list if exist
			let mut entity_issuers = Self::entity_issuers(entity_id);
			ensure!(entity_issuers.contains(&issuer_id), "This account is not an issuer of the entity");
			entity_issuers.retain(|x| x == &issuer_id);
			<EntityIssuers<T>>::insert(entity_id, entity_issuers);
		}

		pub fn create_certificate(origin, issuer_id: T::AccountId, nonce: u64, certificate: Vec<u8>) {
			let sender = ensure_signed(origin)?;
			ensure!(<OfficalAccountsIndex<T>>::contains_key(&sender), "You do not have permission to do this opertion!");

			let nonce_on_chain = Self::nonce_by_issuer_id(&issuer_id);
			ensure!(nonce == nonce_on_chain, "Certificate nonce is wrong");

			// parse the certificate to get infos. certificate length is 130, the hex length with 0x is 262
			let version = &certificate[0];
			let entity_id: H256 = H256::from_slice(&certificate[1..33]);
			let hash = &certificate[33..65];
			let signature: &[u8] = &certificate[65..];

			// ensure the entity exists
			ensure!(<Entities>::contains_key(entity_id), "The entity does not existed");

			// ensure the issuer belongs to the entity
			let entity_issuers = Self::entity_issuers(entity_id);
			ensure!(entity_issuers.contains(&issuer_id), "The issuer is not from the entity");

			// ensure the certificate is signed by the issuer
			let mut sig = [0; 64];
			sig.copy_from_slice(signature);
			// sr25519_verify(&sr25519::Signature(sig), hash, &sr25519::Public(issuer_id));

			let certificates_count = Self::certificates_count();
			let new_certificates_count = certificates_count.checked_add(1).ok_or("Overflow adding a new certificate to total supply")?;
			let new_nonce = nonce.checked_add(1).ok_or("Overflow inc nonce")?;

			// add the new certificate to list
			<CertificatesArray>::insert(certificates_count, certificate.clone());
			<CertificatesCount>::put(new_certificates_count);
			<CertificatesIndex>::insert(certificate.clone(), certificates_count);

			// update the entity of this certificate
			<EntityOfCertificate>::insert(certificate, entity_id);

			// update the nonce
			<NonceByIssuerId<T>>::insert(issuer_id, new_nonce);
		}
	}
}
