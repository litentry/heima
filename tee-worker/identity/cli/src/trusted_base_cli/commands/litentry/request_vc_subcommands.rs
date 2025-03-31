use crate::{
	trusted_base_cli::commands::litentry::request_vc::{to_chains, to_para_str},
	CliError,
};
use ita_stf::Assertion;
use litentry_hex_utils::decode_hex;
use litentry_primitives::{
	AchainableAmount, AchainableAmountHolding, AchainableAmountToken, AchainableAmounts,
	AchainableBasic, AchainableBetweenPercents, AchainableClassOfYear, AchainableDate,
	AchainableDateInterval, AchainableDatePercent, AchainableParams, AchainableToken,
	BnbDigitDomainType, ContestType, DynamicContractParams, DynamicParams, EVMTokenType,
	GenericDiscordRoleType, OneBlockCourseType, PlatformUserType, SoraQuizType,
	VIP3MembershipCardLevel, Web3NftType, Web3TokenType,
};
use sp_core::H160;

// see `assertion.rs`
#[derive(Subcommand, Debug)]
pub enum Command {
	A1,
	A2(A2Arg),
	A3(A3Arg),
	A4(HolderArg),
	A6,
	A7(HolderArg),
	A8(A8Arg),
	A10(HolderArg),
	A11(HolderArg),
	A13(A13Arg),
	A14,
	#[clap(subcommand)]
	Achainable(AchainableCommand),
	A20,
	#[clap(subcommand)]
	OneBlock(OneblockCommand),
	#[clap(subcommand)]
	GenericDiscordRole(GenericDiscordRoleCommand),
	BnbDomainHolding,
	#[clap(subcommand)]
	BnbDigitalDomainClub(BnbDigitalDomainClubCommand),
	#[clap(subcommand)]
	VIP3MembershipCard(VIP3MembershipCardLevelCommand),
	WeirdoGhostGangHolder,
	LITStaking,
	#[clap(subcommand)]
	EVMAmountHolding(EVMAmountHoldingCommand),
	BRC20AmountHolder,
	CryptoSummary,
	#[clap(subcommand)]
	TokenHoldingAmount(TokenHoldingAmountCommand),
	#[clap(subcommand)]
	PlatformUser(PlatformUserCommand),
	#[clap(subcommand)]
	NftHolder(NftHolderCommand),
	Dynamic(DynamicArg),
	LinkedIdentities,
}

#[derive(Args, Debug)]
pub struct A2Arg {
	pub guild_id: String,
}

#[derive(Args, Debug)]
pub struct DynamicArg {
	// hex encoded smart contract id
	pub smart_contract_id: String,
	// hex encoded smart contract params
	// can use this online tool to encode params: https://abi.hashex.org/
	pub smart_contract_param: Option<String>,
	pub return_log: Option<bool>,
}

#[derive(Args, Debug)]
pub struct A3Arg {
	pub guild_id: String,
	pub channel_id: String,
	pub role_id: String,
}

// used in A4/A7/A10/A11
#[derive(Args, Debug)]
pub struct HolderArg {
	pub minimum_amount: String,
}

#[derive(Args, Debug)]
pub struct A8Arg {
	#[clap(num_args = 0.., value_delimiter = ',')]
	pub networks: Vec<String>,
}

#[derive(Args, Debug)]
pub struct A13Arg {
	pub account: String,
}

#[derive(Subcommand, Debug)]
pub enum AchainableCommand {
	AmountHolding(AmountHoldingArg),
	AmountToken(AmountTokenArg),
	Amount(AmountArg),
	Amounts(AmountsArg),
	Basic(BasicArg),
	BetweenPercents(BetweenPercentsArg),
	ClassOfYear(ClassOfYearArg),
	DateInterval(DateIntervalArg),
	DatePercent(DatePercentArg),
	Date(DateArg),
	Token(TokenArg),
}

#[derive(Subcommand, Debug)]
pub enum OneblockCommand {
	Completion,
	Outstanding,
	Participation,
}

#[derive(Subcommand, Debug)]
pub enum GenericDiscordRoleCommand {
	#[clap(subcommand)]
	Contest(ContestCommand),
	#[clap(subcommand)]
	SoraQuiz(SoraQuizCommand),
}

#[derive(Subcommand, Debug)]
pub enum ContestCommand {
	Legend,
	Popularity,
	Participant,
}

#[derive(Subcommand, Debug)]
pub enum SoraQuizCommand {
	Attendee,
	Master,
}

#[derive(Subcommand, Debug)]
pub enum BnbDigitalDomainClubCommand {
	Bnb999ClubMember,
	Bnb10kClubMember,
}

#[derive(Subcommand, Debug)]
pub enum VIP3MembershipCardLevelCommand {
	Gold,
	Silver,
}

#[derive(Subcommand, Debug)]
pub enum EVMAmountHoldingCommand {
	Ton,
	Trx,
}

#[derive(Subcommand, Debug)]
pub enum TokenHoldingAmountCommand {
	Bnb,
	Eth,
	SpaceId,
	Lit,
	Wbtc,
	Usdc,
	Usdt,
	Crv,
	Matic,
	Dydx,
	Amp,
	Cvx,
	Tusd,
	Usdd,
	Gusd,
	Link,
	Grt,
	Comp,
	People,
	Gtc,
	Ton,
	Trx,
	Nfp,
	Sol,
	Mcrt,
	Btc,
	Bean,
	An,
	Tuna,
}

#[derive(Subcommand, Debug)]
pub enum PlatformUserCommand {
	KaratDao,
	MagicCraftStaking,
	DarenMarket,
}

#[derive(Subcommand, Debug)]
pub enum NftHolderCommand {
	WeirdoGhostGang,
	Club3Sbt,
	MFan,
	Mvp,
}

// positional args (to vec) + required arg + optional arg is a nightmare combination for clap parser,
// additionally, only the last positional argument, or second to last positional argument may be set to `.num_args()`
//
// the best bet is to use a flag explicitly, be sure to use equal form for `chain`, e.g.:
// -- name -c=bsc,ethereum 10
// -- name -c=bsc,ethereum 10 token
macro_rules! AchainableCommandArgs {
	($type_name:ident, {$( $field_name:ident : $field_type:ty , )* }) => {
		#[derive(Args, Debug)]
		pub struct $type_name {
			pub name: String,
			#[clap(
				short, long,
				num_args = 1..,
				required = true,
				value_delimiter = ',',
			)]
			pub chain: Vec<String>,
			$( pub $field_name: $field_type ),*
		}
	};
}

AchainableCommandArgs!(AmountHoldingArg, {
	amount: String,
	date: String,
	token: Option<String>,
});

AchainableCommandArgs!(AmountTokenArg, {
	amount: String,
	token: Option<String>,
});

AchainableCommandArgs!(AmountArg, {
	amount: String,
});

AchainableCommandArgs!(AmountsArg, {
	amount1: String,
	amount2: String,
});

AchainableCommandArgs!(BasicArg, {});

AchainableCommandArgs!(BetweenPercentsArg, {
	greater_than_or_equal_to: String,
	less_than_or_equal_to: String,
});

AchainableCommandArgs!(ClassOfYearArg, {});

AchainableCommandArgs!(DateIntervalArg, {
	start_date: String,
	end_date: String,
});

AchainableCommandArgs!(DatePercentArg, {
	token: String,
	date: String,
	percent: String,
});

AchainableCommandArgs!(DateArg, {
	date: String,
});

AchainableCommandArgs!(TokenArg, {
	token: String,
});

impl Command {
	// helper fn to convert a `Command` to `Assertion`
	pub fn to_assertion(&self) -> Result<Assertion, CliError> {
		use Assertion::*;
		match self {
			Command::A1 => Ok(A1),
			Command::A2(arg) => Ok(A2(to_para_str(&arg.guild_id))),
			Command::A3(arg) => Ok(A3(
				to_para_str(&arg.guild_id),
				to_para_str(&arg.channel_id),
				to_para_str(&arg.role_id),
			)),
			Command::A4(arg) => Ok(A4(to_para_str(&arg.minimum_amount))),
			Command::A6 => Ok(A6),
			Command::A7(arg) => Ok(A7(to_para_str(&arg.minimum_amount))),
			Command::A8(arg) => Ok(A8(to_chains(&arg.networks))),
			Command::A10(arg) => Ok(A10(to_para_str(&arg.minimum_amount))),
			Command::A11(arg) => Ok(A11(to_para_str(&arg.minimum_amount))),
			Command::A13(arg) => {
				let raw: [u8; 32] = decode_hex(&arg.account).unwrap().try_into().unwrap();
				Ok(A13(raw.into()))
			},
			Command::A14 => Ok(A14),
			Command::Achainable(c) => Ok(match c {
				AchainableCommand::AmountHolding(arg) => {
					Achainable(AchainableParams::AmountHolding(AchainableAmountHolding {
						name: to_para_str(&arg.name),
						chain: to_chains(&arg.chain),
						amount: to_para_str(&arg.amount),
						date: to_para_str(&arg.date),
						token: arg.token.as_ref().map(to_para_str),
					}))
				},
				AchainableCommand::AmountToken(arg) => {
					Achainable(AchainableParams::AmountToken(AchainableAmountToken {
						name: to_para_str(&arg.name),
						chain: to_chains(&arg.chain),
						amount: to_para_str(&arg.amount),
						token: arg.token.as_ref().map(to_para_str),
					}))
				},
				AchainableCommand::Amount(arg) => {
					Achainable(AchainableParams::Amount(AchainableAmount {
						name: to_para_str(&arg.name),
						chain: to_chains(&arg.chain),
						amount: to_para_str(&arg.amount),
					}))
				},
				AchainableCommand::Amounts(arg) => {
					Achainable(AchainableParams::Amounts(AchainableAmounts {
						name: to_para_str(&arg.name),
						chain: to_chains(&arg.chain),
						amount1: to_para_str(&arg.amount1),
						amount2: to_para_str(&arg.amount2),
					}))
				},
				AchainableCommand::Basic(arg) => {
					Achainable(AchainableParams::Basic(AchainableBasic {
						name: to_para_str(&arg.name),
						chain: to_chains(&arg.chain),
					}))
				},
				AchainableCommand::BetweenPercents(arg) => {
					Achainable(AchainableParams::BetweenPercents(AchainableBetweenPercents {
						name: to_para_str(&arg.name),
						chain: to_chains(&arg.chain),
						greater_than_or_equal_to: to_para_str(&arg.greater_than_or_equal_to),
						less_than_or_equal_to: to_para_str(&arg.less_than_or_equal_to),
					}))
				},
				AchainableCommand::ClassOfYear(arg) => {
					Achainable(AchainableParams::ClassOfYear(AchainableClassOfYear {
						name: to_para_str(&arg.name),
						chain: to_chains(&arg.chain),
					}))
				},
				AchainableCommand::DateInterval(arg) => {
					Achainable(AchainableParams::DateInterval(AchainableDateInterval {
						name: to_para_str(&arg.name),
						chain: to_chains(&arg.chain),
						start_date: to_para_str(&arg.start_date),
						end_date: to_para_str(&arg.end_date),
					}))
				},
				AchainableCommand::DatePercent(arg) => {
					Achainable(AchainableParams::DatePercent(AchainableDatePercent {
						name: to_para_str(&arg.name),
						chain: to_chains(&arg.chain),
						date: to_para_str(&arg.date),
						percent: to_para_str(&arg.percent),
						token: to_para_str(&arg.token),
					}))
				},
				AchainableCommand::Date(arg) => {
					Achainable(AchainableParams::Date(AchainableDate {
						name: to_para_str(&arg.name),
						chain: to_chains(&arg.chain),
						date: to_para_str(&arg.date),
					}))
				},
				AchainableCommand::Token(arg) => {
					Achainable(AchainableParams::Token(AchainableToken {
						name: to_para_str(&arg.name),
						chain: to_chains(&arg.chain),
						token: to_para_str(&arg.token),
					}))
				},
			}),
			Command::A20 => Ok(A20),
			Command::OneBlock(c) => Ok(match c {
				OneblockCommand::Completion => OneBlock(OneBlockCourseType::CourseCompletion),
				OneblockCommand::Outstanding => OneBlock(OneBlockCourseType::CourseOutstanding),
				OneblockCommand::Participation => OneBlock(OneBlockCourseType::CourseParticipation),
			}),
			Command::GenericDiscordRole(c) => Ok(match c {
				GenericDiscordRoleCommand::Contest(s) => match s {
					ContestCommand::Legend => {
						GenericDiscordRole(GenericDiscordRoleType::Contest(ContestType::Legend))
					},
					ContestCommand::Popularity => {
						GenericDiscordRole(GenericDiscordRoleType::Contest(ContestType::Popularity))
					},
					ContestCommand::Participant => GenericDiscordRole(
						GenericDiscordRoleType::Contest(ContestType::Participant),
					),
				},
				GenericDiscordRoleCommand::SoraQuiz(s) => match s {
					SoraQuizCommand::Attendee => {
						GenericDiscordRole(GenericDiscordRoleType::SoraQuiz(SoraQuizType::Attendee))
					},
					SoraQuizCommand::Master => {
						GenericDiscordRole(GenericDiscordRoleType::SoraQuiz(SoraQuizType::Master))
					},
				},
			}),
			Command::BnbDomainHolding => Ok(BnbDomainHolding),
			Command::BnbDigitalDomainClub(c) => Ok(match c {
				BnbDigitalDomainClubCommand::Bnb999ClubMember => {
					BnbDigitDomainClub(BnbDigitDomainType::Bnb999ClubMember)
				},
				BnbDigitalDomainClubCommand::Bnb10kClubMember => {
					BnbDigitDomainClub(BnbDigitDomainType::Bnb10kClubMember)
				},
			}),
			Command::VIP3MembershipCard(arg) => Ok(match arg {
				VIP3MembershipCardLevelCommand::Gold => {
					VIP3MembershipCard(VIP3MembershipCardLevel::Gold)
				},
				VIP3MembershipCardLevelCommand::Silver => {
					VIP3MembershipCard(VIP3MembershipCardLevel::Silver)
				},
			}),
			Command::WeirdoGhostGangHolder => Ok(WeirdoGhostGangHolder),
			Command::EVMAmountHolding(c) => Ok(match c {
				EVMAmountHoldingCommand::Ton => EVMAmountHolding(EVMTokenType::Ton),
				EVMAmountHoldingCommand::Trx => EVMAmountHolding(EVMTokenType::Trx),
			}),
			Command::CryptoSummary => Ok(CryptoSummary),
			Command::LITStaking => Ok(LITStaking),
			Command::BRC20AmountHolder => Ok(BRC20AmountHolder),
			Command::TokenHoldingAmount(arg) => Ok(match arg {
				TokenHoldingAmountCommand::Bnb => TokenHoldingAmount(Web3TokenType::Bnb),
				TokenHoldingAmountCommand::Eth => TokenHoldingAmount(Web3TokenType::Eth),
				TokenHoldingAmountCommand::SpaceId => TokenHoldingAmount(Web3TokenType::SpaceId),
				TokenHoldingAmountCommand::Lit => TokenHoldingAmount(Web3TokenType::Lit),
				TokenHoldingAmountCommand::Wbtc => TokenHoldingAmount(Web3TokenType::Wbtc),
				TokenHoldingAmountCommand::Usdc => TokenHoldingAmount(Web3TokenType::Usdc),
				TokenHoldingAmountCommand::Usdt => TokenHoldingAmount(Web3TokenType::Usdt),
				TokenHoldingAmountCommand::Crv => TokenHoldingAmount(Web3TokenType::Crv),
				TokenHoldingAmountCommand::Matic => TokenHoldingAmount(Web3TokenType::Matic),
				TokenHoldingAmountCommand::Dydx => TokenHoldingAmount(Web3TokenType::Dydx),
				TokenHoldingAmountCommand::Amp => TokenHoldingAmount(Web3TokenType::Amp),
				TokenHoldingAmountCommand::Cvx => TokenHoldingAmount(Web3TokenType::Cvx),
				TokenHoldingAmountCommand::Tusd => TokenHoldingAmount(Web3TokenType::Tusd),
				TokenHoldingAmountCommand::Usdd => TokenHoldingAmount(Web3TokenType::Usdd),
				TokenHoldingAmountCommand::Gusd => TokenHoldingAmount(Web3TokenType::Gusd),
				TokenHoldingAmountCommand::Link => TokenHoldingAmount(Web3TokenType::Link),
				TokenHoldingAmountCommand::Grt => TokenHoldingAmount(Web3TokenType::Grt),
				TokenHoldingAmountCommand::Comp => TokenHoldingAmount(Web3TokenType::Comp),
				TokenHoldingAmountCommand::People => TokenHoldingAmount(Web3TokenType::People),
				TokenHoldingAmountCommand::Gtc => TokenHoldingAmount(Web3TokenType::Gtc),
				TokenHoldingAmountCommand::Ton => TokenHoldingAmount(Web3TokenType::Ton),
				TokenHoldingAmountCommand::Trx => TokenHoldingAmount(Web3TokenType::Trx),
				TokenHoldingAmountCommand::Nfp => TokenHoldingAmount(Web3TokenType::Nfp),
				TokenHoldingAmountCommand::Sol => TokenHoldingAmount(Web3TokenType::Sol),
				TokenHoldingAmountCommand::Mcrt => TokenHoldingAmount(Web3TokenType::Mcrt),
				TokenHoldingAmountCommand::Btc => TokenHoldingAmount(Web3TokenType::Btc),
				TokenHoldingAmountCommand::Bean => TokenHoldingAmount(Web3TokenType::Bean),
				TokenHoldingAmountCommand::An => TokenHoldingAmount(Web3TokenType::An),
				TokenHoldingAmountCommand::Tuna => TokenHoldingAmount(Web3TokenType::Tuna),
			}),
			Command::PlatformUser(arg) => Ok(match arg {
				PlatformUserCommand::KaratDao => PlatformUser(PlatformUserType::KaratDao),
				PlatformUserCommand::MagicCraftStaking => {
					PlatformUser(PlatformUserType::MagicCraftStaking)
				},
				PlatformUserCommand::DarenMarket => PlatformUser(PlatformUserType::DarenMarket),
			}),
			Command::NftHolder(arg) => Ok(match arg {
				NftHolderCommand::WeirdoGhostGang => NftHolder(Web3NftType::WeirdoGhostGang),
				NftHolderCommand::Club3Sbt => NftHolder(Web3NftType::Club3Sbt),
				NftHolderCommand::MFan => NftHolder(Web3NftType::MFan),
				NftHolderCommand::Mvp => NftHolder(Web3NftType::Mvp),
			}),
			Command::Dynamic(arg) => {
				let decoded_id = hex::decode(&arg.smart_contract_id.clone()).unwrap();
				let id_bytes: [u8; 20] = decoded_id.try_into().unwrap();

				let smart_contract_params = match &arg.smart_contract_param {
					Some(p) => {
						let params = hex::decode(p).unwrap();
						let params_len = params.len();
						let truncated_params = DynamicContractParams::truncate_from(params);
						let truncated_params_len = truncated_params.len();
						if params_len > truncated_params_len {
							println!(
								"The dynamic params length {} is over the maximum value {}",
								params_len, truncated_params_len
							);
							Err(CliError::Extrinsic {
								msg: format!(
									"The dynamic params length {} is over the maximum value {}",
									params_len, truncated_params_len
								),
							})
						} else {
							Ok(Some(truncated_params))
						}
					},
					None => Ok(None),
				}?;

				Ok(Assertion::Dynamic(DynamicParams {
					smart_contract_id: H160::from(id_bytes),
					smart_contract_params,
					return_log: arg.return_log.unwrap_or_default(),
				}))
			},
			Command::LinkedIdentities => Ok(LinkedIdentities),
		}
	}
}
