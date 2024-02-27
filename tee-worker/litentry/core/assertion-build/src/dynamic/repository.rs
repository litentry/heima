#[cfg(all(feature = "std", feature = "sgx"))]
compile_error!("feature \"std\" and feature \"sgx\" cannot be enabled at the same time");

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate sgx_tstd as std;

#[cfg(all(not(feature = "std"), feature = "sgx"))]
extern crate hex_sgx as hex;

use primitive_types::H160;
use std::{collections::HashMap, vec::Vec};

pub trait SmartContractRepository {
	fn get(&self, id: &H160) -> Option<Vec<u8>>;
}

pub struct InMemorySmartContractRepo {
	map: HashMap<H160, Vec<u8>>,
}

impl InMemorySmartContractRepo {
	pub fn new() -> Self {
		let mut map = HashMap::new();
		//a1
		map.insert(
            hash(0),
            hex::decode("608060405234801561001057600080fd5b5061077f806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c806309c5eabe1461003b578063c66e70f014610067575b600080fd5b61004e6100493660046103cd565b61008a565b60405161005e9493929190610466565b60405180910390f35b61007a6100753660046104c8565b6100c1565b604051901515815260200161005e565b6060806060600080858060200190518101906100a69190610548565b90506100b1816100e6565b9450945094509450509193509193565b6000816000015163ffffffff16600414156100de57506001919050565b506000919050565b60608060606000806040518060800160405280604581526020016106ce60459139905060006040518060400160405280601b81526020017f4261736963204964656e7469747920566572696669636174696f6e00000000008152509050600060405180606001604052806037815260200161071360379139905060008080805b8b518110156101dd576101918c82815181106101845761018461068e565b60200260200101516101fc565b1561019f57600191506101cb565b6101c18c82815181106101b4576101b461068e565b602002602001015161022b565b156101cb57600192505b806101d5816106a4565b915050610166565b508080156101e85750815b959b949a5092985093965091945050505050565b600061020782610254565b8061021657506102168261026b565b80610225575061022582610288565b92915050565b6000610236826102a5565b806102455750610245826100c1565b806102255750610225826102c2565b805160009063ffffffff166100de57506001919050565b6000816000015163ffffffff16600114156100de57506001919050565b6000816000015163ffffffff16600214156100de57506001919050565b6000816000015163ffffffff16600314156100de57506001919050565b6000816000015163ffffffff16600514156100de57506001919050565b634e487b7160e01b600052604160045260246000fd5b6040805190810167ffffffffffffffff81118282101715610318576103186102df565b60405290565b604051601f8201601f1916810167ffffffffffffffff81118282101715610347576103476102df565b604052919050565b600067ffffffffffffffff821115610369576103696102df565b50601f01601f191660200190565b600082601f83011261038857600080fd5b813561039b6103968261034f565b61031e565b8181528460208386010111156103b057600080fd5b816020850160208301376000918101602001919091529392505050565b6000602082840312156103df57600080fd5b813567ffffffffffffffff8111156103f657600080fd5b61040284828501610377565b949350505050565b60005b8381101561042557818101518382015260200161040d565b83811115610434576000848401525b50505050565b6000815180845261045281602086016020860161040a565b601f01601f19169290920160200192915050565b608081526000610479608083018761043a565b828103602084015261048b818761043a565b9050828103604084015261049f818661043a565b915050821515606083015295945050505050565b63ffffffff811681146104c557600080fd5b50565b6000602082840312156104da57600080fd5b813567ffffffffffffffff808211156104f257600080fd5b908301906040828603121561050657600080fd5b61050e6102f5565b8235610519816104b3565b815260208301358281111561052d57600080fd5b61053987828601610377565b60208301525095945050505050565b6000602080838503121561055b57600080fd5b825167ffffffffffffffff8082111561057357600080fd5b818501915085601f83011261058757600080fd5b815181811115610599576105996102df565b8060051b6105a885820161031e565b91825283810185019185810190898411156105c257600080fd5b86860192505b83831015610681578251858111156105e05760008081fd5b86016040818c03601f19018113156105f85760008081fd5b6106006102f5565b8983015161060d816104b3565b815282820151888111156106215760008081fd5b8084019350508c603f8401126106375760008081fd5b898301516106476103968261034f565b8181528e8483870101111561065c5760008081fd5b61066b828d830186880161040a565b828c0152508452505091860191908601906105c8565b9998505050505050505050565b634e487b7160e01b600052603260045260246000fd5b60006000198214156106c657634e487b7160e01b600052601160045260246000fd5b506001019056fe596f75277665206964656e746966696564206174206c65617374206f6e65206163636f756e742f6164647265737320696e20626f7468205765623220616e6420576562332e246861735f776562325f6163636f756e74203d3d207472756520616e6420246861735f776562335f6163636f756e74203d3d2074727565a2646970667358221220b9e0a3709e61b24f25c55fb119105235f53f0d3cd495eb458fb49d8fd6ca00c764736f6c63430008080033").unwrap()
        );
		//a20
		map.insert(
			hash(1),
			hex::decode("608060405234801561001057600080fd5b50610954806100206000396000f3fe608060405234801561001057600080fd5b50600436106100415760003560e01c806309c5eabe14610046578063a72fb55e14610072578063c66e70f014610092575b600080fd5b61005961005436600461041e565b6100b5565b60405161006994939291906104b7565b60405180910390f35b610085610080366004610504565b6100ec565b6040516100699190610568565b6100a56100a0366004610597565b610118565b6040519015158152602001610069565b6060806060600080858060200190518101906100d19190610617565b90506100dc8161013d565b9450945094509450509193509193565b6060828260405160200161010192919061075d565b604051602081830303815290604052905092915050565b6000816000015163ffffffff166004141561013557506001919050565b506000919050565b60608060606000806040518060c001604052806086815260200161085160869139604080518082018252601c81527f49444875622045564d2056657273696f6e204561726c7920426972640000000060208083019190915282518084019093526013835272246861735f6a6f696e6564203d3d207472756560681b908301529192506000805b89518110156102b0576101ee8a82815181106101e1576101e161078b565b60200260200101516102c0565b156102455760006040518060c00160405280608681526020016107cb6086913960408051808201909152600a8152690bda185cd29bda5b995960b21b602082015290915061023c82826102d7565b93505050610293565b60006040518060800160405280604881526020016108d76048913960408051808201909152600a8152690bda185cd29bda5b995960b21b602082015290915061028e82826102d7565b935050505b811561029e576102b0565b806102a8816107a1565b9150506101c3565b5092989197509550909350915050565b805160009063ffffffff1661013557506001919050565b600080600084846040516020016102ef92919061075d565b60408051601f19818403018152908290528051909250906020818382860160006003600019f161031e57600080fd5b60208101604052519695505050505050565b634e487b7160e01b600052604160045260246000fd5b6040805190810167ffffffffffffffff8111828210171561036957610369610330565b60405290565b604051601f8201601f1916810167ffffffffffffffff8111828210171561039857610398610330565b604052919050565b600067ffffffffffffffff8211156103ba576103ba610330565b50601f01601f191660200190565b600082601f8301126103d957600080fd5b81356103ec6103e7826103a0565b61036f565b81815284602083860101111561040157600080fd5b816020850160208301376000918101602001919091529392505050565b60006020828403121561043057600080fd5b813567ffffffffffffffff81111561044757600080fd5b610453848285016103c8565b949350505050565b60005b8381101561047657818101518382015260200161045e565b83811115610485576000848401525b50505050565b600081518084526104a381602086016020860161045b565b601f01601f19169290920160200192915050565b6080815260006104ca608083018761048b565b82810360208401526104dc818761048b565b905082810360408401526104f0818661048b565b915050821515606083015295945050505050565b6000806040838503121561051757600080fd5b823567ffffffffffffffff8082111561052f57600080fd5b61053b868387016103c8565b9350602085013591508082111561055157600080fd5b5061055e858286016103c8565b9150509250929050565b60208152600061057b602083018461048b565b9392505050565b63ffffffff8116811461059457600080fd5b50565b6000602082840312156105a957600080fd5b813567ffffffffffffffff808211156105c157600080fd5b90830190604082860312156105d557600080fd5b6105dd610346565b82356105e881610582565b81526020830135828111156105fc57600080fd5b610608878286016103c8565b60208301525095945050505050565b6000602080838503121561062a57600080fd5b825167ffffffffffffffff8082111561064257600080fd5b818501915085601f83011261065657600080fd5b81518181111561066857610668610330565b8060051b61067785820161036f565b918252838101850191858101908984111561069157600080fd5b86860192505b83831015610750578251858111156106af5760008081fd5b86016040818c03601f19018113156106c75760008081fd5b6106cf610346565b898301516106dc81610582565b815282820151888111156106f05760008081fd5b8084019350508c603f8401126107065760008081fd5b898301516107166103e7826103a0565b8181528e8483870101111561072b5760008081fd5b61073a828d830186880161045b565b828c015250845250509186019190860190610697565b9998505050505050505050565b604081526000610770604083018561048b565b8281036020840152610782818561048b565b95945050505050565b634e487b7160e01b600052603260045260246000fd5b60006000198214156107c357634e487b7160e01b600052601160045260246000fd5b506001019056fe687474703a2f2f6c6f63616c686f73743a31393532372f6576656e74732f646f65732d757365722d6a6f696e65642d65766d2d63616d706169676e3f6163636f756e743d307864343335393363373135666464333163363131343161626430346139396664363832326338353538383534636364653339613536383465376135366461323764546865207573657220697320616e206561726c7920626972642075736572206f6620746865204964656e746974794875622045564d2076657273696f6e20616e64206861732067656e657261746564206174206c6561737420312063726564656e7469616c20647572696e672032303233204175672031347468207e2041756720323173742e687474703a2f2f6c6f63616c686f73743a31393532372f6576656e74732f646f65732d757365722d6a6f696e65642d65766d2d63616d706169676e3f6163636f756e743d74657374a26469706673582212200f6ff89169aba3f04aadacae961022fab904e7df24cef7ed40057a33beccec3564736f6c63430008080033").unwrap()
		);
		// is age over 50
		map.insert(
            hash(2),
            hex::decode("608060405234801561001057600080fd5b5061063d806100206000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c806309c5eabe1461003b578063c66e70f014610067575b600080fd5b61004e6100493660046102f2565b61008a565b60405161005e949392919061038b565b60405180910390f35b61007a6100753660046103ed565b6100c1565b604051901515815260200161005e565b6060806060600080858060200190518101906100a6919061046d565b90506100b1816100e6565b9450945094509450509193509193565b6000816000015163ffffffff16600414156100de57506001919050565b506000919050565b60608060606000806040518060600160405280603181526020016105b46031913990506000604051806040016040528060148152602001732f646174612f332f656d706c6f7965655f61676560601b815250905060006040518060600160405280602381526020016105e560239139604080518082018252600a81526904973206f7665722035360b41b602080830191909152825180840190935260088352670616765203e2035360c41b908301529192506000806101a587876101d3565b905060328160070b13156101bc57600191506101c1565b600091505b50929a91995097509095509350505050565b60405160009081908385016020828288866002600019f16101f357600080fd5b506020810160405251949350505050565b634e487b7160e01b600052604160045260246000fd5b6040805190810167ffffffffffffffff8111828210171561023d5761023d610204565b60405290565b604051601f8201601f1916810167ffffffffffffffff8111828210171561026c5761026c610204565b604052919050565b600067ffffffffffffffff82111561028e5761028e610204565b50601f01601f191660200190565b600082601f8301126102ad57600080fd5b81356102c06102bb82610274565b610243565b8181528460208386010111156102d557600080fd5b816020850160208301376000918101602001919091529392505050565b60006020828403121561030457600080fd5b813567ffffffffffffffff81111561031b57600080fd5b6103278482850161029c565b949350505050565b60005b8381101561034a578181015183820152602001610332565b83811115610359576000848401525b50505050565b6000815180845261037781602086016020860161032f565b601f01601f19169290920160200192915050565b60808152600061039e608083018761035f565b82810360208401526103b0818761035f565b905082810360408401526103c4818661035f565b915050821515606083015295945050505050565b63ffffffff811681146103ea57600080fd5b50565b6000602082840312156103ff57600080fd5b813567ffffffffffffffff8082111561041757600080fd5b908301906040828603121561042b57600080fd5b61043361021a565b823561043e816103d8565b815260208301358281111561045257600080fd5b61045e8782860161029c565b60208301525095945050505050565b6000602080838503121561048057600080fd5b825167ffffffffffffffff8082111561049857600080fd5b818501915085601f8301126104ac57600080fd5b8151818111156104be576104be610204565b8060051b6104cd858201610243565b91825283810185019185810190898411156104e757600080fd5b86860192505b838310156105a6578251858111156105055760008081fd5b86016040818c03601f190181131561051d5760008081fd5b61052561021a565b89830151610532816103d8565b815282820151888111156105465760008081fd5b8084019350508c603f84011261055c5760008081fd5b8983015161056c6102bb82610274565b8181528e848387010111156105815760008081fd5b610590828d830186880161032f565b828c0152508452505091860191908601906104ed565b999850505050505050505056fe68747470733a2f2f64756d6d792e726573746170696578616d706c652e636f6d2f6170692f76312f656d706c6f7965657349732074686520656d706c6f796565206f766572203530207965617273206f6c64203fa26469706673582212209afbbff9c7067b373d283c1dbf0c57514188d257106c6e490b56ad72047a5cfa64736f6c63430008080033").unwrap()
        );
		InMemorySmartContractRepo { map }
	}
}

impl Default for InMemorySmartContractRepo {
	fn default() -> Self {
		Self::new()
	}
}

impl SmartContractRepository for InMemorySmartContractRepo {
	fn get(&self, id: &H160) -> Option<Vec<u8>> {
		self.map.get(id).cloned()
	}
}

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}
