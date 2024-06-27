---
title: AVM 架构
abbrlink: 6cd0a982
date: 2024-06-27 09:00:00
---

## 核心指令

OP_DECODEBLOCKINFO
OP_FT_BALANCE_ADD
OP_FT_BALANCE
OP_FT_COUNT
OP_FT_ITEM
OP_FT_WITHDRAW
OP_GETBLOCKINFO
OP_HASH_FN
OP_KV_DELETE
OP_KV_EXISTS
OP_KV_GET
OP_KV_PUT
OP_NFT_COUNT
OP_NFT_EXISTS
OP_NFT_ITEM
OP_NFT_PUT
OP_NFT_WITHDRAW

# 整体架构

![arch](/article_photo/arch.png)

左侧为BTC主网，里面包含了我们的合约数据、状态数据等；

中间部分为基于Indexer的编程部分，我们可以调用智能合约代码，代码可以是通过高级语言编译而成的，当执行合约之后，相关的数据（包含token数据，状态数据）在本地进行存储。

具体执行的函数形式包含两类：一类是Btc原有的op-code，比如 OP_ADD, OP_MUL 等，这里还出现了还未被通过的OP_CAT，另一类是开发者自定义的op-code，OP_NFT_EXISTS, OP_FT_COUN 等。开发者可以在官方库`avm-interpreter`的基础上继承然后开发新的函数。

对于自定义的op-code，提出的Two Stack PDA是图灵完备的。

这里的高级语言开发合约部分，用的是sCrypt来进行开发的。[sCrypt](https://docs.scrypt.io/)是一门基于TypeScript的DSL。可以用来在BSV上面写智能合约，这个语言也被证明是图灵完备的。

# 构造流程

要执行AVM，必须要先编写合约到链上。从代码里面可以看出，我们在parse_operation_from_scrip方法里面新加了三个Operation，分别是def，new，还有c。def是定一个PROTOCOL到链上。new是new一个CONTRACT。c是call，是合约的调用执行。

```
elif op_found_struct['op'] == 'def' and op_found_struct['input_index'] == 0:
    mint_info['type'] = 'PROTOCOL'
    # With AVMFactory the control fields are in the top level payload not the mint_info
    # The reason is basically to simplify and optimize definitions
    protocol = payload.get('p')
    if isinstance(protocol, str) and protocol == '':
        logger.warning(f'AVMFactory protocol name is invalid detected empty str {hash_to_hex_str(tx_hash)}. Skipping....')
        return None, None
    logger.debug(f'NFT request_protocol protocol name p {hash_to_hex_str(tx_hash)}, {protocol} {mint_info}')
    if not isinstance(protocol, str) or not is_valid_protocol_string_name(protocol):
        logger.warning(f'NFT request_protocol name p is invalid {hash_to_hex_str(tx_hash)}, {protocol} {mint_info}. Skipping....')
        return None, None
    mint_info['$request_protocol'] = protocol
    # TODO: Perform sanity checks on the payload here...
elif op_found_struct['op'] == 'new' and op_found_struct['input_index'] == 0:
    mint_info['type'] = 'CONTRACT'
    # With AVMFactory the control fields are in the top level payload not the mint_info
    # The reason is basically to simplify and optimize definitions
    contract_name = payload.get('name')
    if isinstance(contract_name, str) and contract_name == '':
        logger.warning(f'AVMFactory contract_name is invalid detected empty str {hash_to_hex_str(tx_hash)}. Skipping....')
        return None, None
    logger.debug(f'CONTRACT name {hash_to_hex_str(tx_hash)}, {contract_name} {mint_info}')
    # Contract name can be empty
    if isinstance(contract_name, str) and not is_valid_contract_string_name(contract_name):
        logger.warning(f'CONTRACT name is invalid {hash_to_hex_str(tx_hash)}, {contract_name} {mint_info}. Skipping....')
        return None, None
    # If contract name is set then assign request_contract
    if contract_name:
        mint_info['$request_contract'] = contract_name
    protocol_name = payload.get('p')
    if not isinstance(protocol_name, str) or not is_valid_protocol_string_name(protocol_name):
        logger.warning(f'CONTRACT p is invalid {hash_to_hex_str(tx_hash)}, {protocol_name} {mint_info}. Skipping....')
        return None, None
    mint_info['$instance_of_protocol'] = protocol_name
```

# 调用流程

有了合约，那么就需要在同步区块的时候，对tx进行解析并且进行执行。如果indexer在执行的过程中，发现有`c`操作码，想要执行合约。那么就需要把相关合约都给拉出来，创建好上下文，然后执行。

先是引入了一个大类 `CScript`。这是一个bytes的子类，因此只要接受bytes就可以直接使用它。
字节而不是操作码。选择这种格式是为了提高效率，以便一般情况不需要创建很多小的 CScriptOp 对象。然而 iter(script) 确实通过操作码进行迭代。

又引入了 `encode_op_pushdata 方法。
encode_op_pushdata 函数根据数据的长度，对数据进行 PUSHDATA 操作编码，并返回相应的字节序列。具体地，它会根据数据长度选择适当的 PUSHDATA 操作码（OP_PUSHDATA1, OP_PUSHDATA2, OP_PUSHDATA4），并将数据长度和数据一起编码。

接着引入 ReactorContext 类, 用于存储和管理有关区块链状态和交易信息的上下文。

包含参数
state_hash: 区块链状态的哈希值。
state: 当前区块链状态。
state_updates: 状态更新列表或字典。
state_deletes: 状态删除列表或字典。
nft_incoming: 传入的不可替代代币（NFT）的列表或字典。
ft_incoming: 传入的可替代代币（FT）的列表或字典。
nft_balances: NFT 余额列表或字典。
nft_balances_updates: NFT 余额更新列表或字典。
ft_balances: FT 余额列表或字典。
ft_balances_updates: FT 余额更新列表或字典。
nft_withdraws: 提取的 NFT 列表或字典。
ft_withdraws: 提取的 FT 列表或字典。


最后来看AVM的主要执行过程:

处理好前面的流程之后，先实例化 ReactorContext，然后维护 request_tx_context 和 blockchain_context 以及 script_context 上下文。最后执行 ConsensusVerifyScriptAvmExecute。

ConsensusVerifyScriptAvmExecute是一个调用外部C语言库的Python代码，利用ctypes库与C库进行交互。

函数步骤
1. 加载外部库：如果 _libatomicals_consensus 变量未定义，则加载外部C语言库 atomicalsconsensus_library。
2. 反序列化交易：利用 CTransaction.deserialize 将原始交易字节反序列化，并确保反序列化后的数据与原始数据一致。拿到request_tx_context.rawtx_bytes赋值tx_data后续使用。
3. 初始化变量：初始化一些用于保存结果的变量，包括状态哈希、最终状态、状态更新、状态删除、FT的balances、NFT的balances、FT 的withdraw、NFT 的withdraw等。
4. 准备 CBOR 数据：将各种上下文数据序列化为 CBOR 格式，准备传递给外部库。
执行共识验证脚本：调用外部库的 atomicalsconsensus_verify_script_avm 函数进行验证和执行，传递所有必要的数据和变量。
5. 检查错误代码：根据 error_code 和 execute_result 的值，判断执行结果，并处理错误或返回更新的 reactor_context。


所以，简单来说 `atomicalsconsensus.py` 只是相当于一个执行器。用来执行 avm-interprter中的代码。

## 探究 atomicalsconsensus_verify_script_avm

avm-interprter 算是一个魔改版本BSV。我们跟随上面的方法调用，继续往下走。

接着来看 `atomicalsconsensus.cpp` 中的 atomicalsconsensus_verify_script_avm 方法。

这个函数 atomicalsconsensus_verify_script_avm 是用于验证基于 Atomicals 共识协议的脚本。它接受许多输入数据，包括锁定脚本、解锁脚本、交易数据、状态数据等，执行验证逻辑，并返回验证结果和更新的状态数据。

函数逻辑是:
1. 设置初始错误状态
2. 复制状态数据
```
std::vector<std::uint8_t> prevStateHashBytes(prevStateHash, prevStateHash + 32);
```
3. 从 CBOR 数据中解析 JSON
```
auto ftState = json::from_cbor(ftStateBytes, true, true, json::cbor_tag_handler_t::error);
```
4. 调用脚本验证函数
```
int result = ::verify_script_avm(lockScript, lockScriptLen, unlockScript, unlockScriptLen, ftState, ftStateIncoming, nftState, nftStateIncoming, contractState, contractExternalState, txTo, txToLen, flags, err, script_err, script_err_op_num, &stateContext);
```
5. 清理和验证最终状态
6. 复制最终状态数据
7. 计算并复制更新的状态哈希:

所以直接跟着看最核心的 `verify_script_avm`:

函数的主要逻辑是先验证flag，有一个verify_flags，不通过就抛出atomicalsconsensus_ERR_INVALID_FLAGS。然后是反序列化。
```
TxInputStream stream(SER_NETWORK, PROTOCOL_VERSION, txTo, txToLen);
CTransaction tx(deserialize, stream);
CTransactionView txView(tx);

if (GetSerializeSize(tx, PROTOCOL_VERSION) != txToLen) {
    return set_error(err, atomicalsconsensus_ERR_TX_SIZE_MISMATCH);
}
```

设置初始错误状态，创建脚本和预计算交易数据，复制状态数据，创建和初始化脚本执行上下文。
接着调用脚本验证函数：
```
ScriptError tempScriptError = ScriptError::OK;
auto error_code = VerifyScriptAvm(unlockSig, spk, TransactionSignatureChecker(&tx, 0, Amount::zero(), txdata), context, state, &tempScriptError, script_err_op_num);
```

GenericTransactionSignatureChecker 用于检查交易的相对锁定时间是否满足条件。具体地，它检查交易输入的序列号（nSequence）与给定的脚本数字（CScriptNum）之间的关系，以确定交易是否可以在当前区块高度或时间被包含在区块中。

VerifyScriptAvm 这个函数的目的是将解锁脚本和锁定脚本结合起来执行，并在成功时返回累积的脚本执行指标。失败时，这些结果不应依赖。

VerifyScriptAvm 函数内部调用 EvalScript 来执行 AVM 的合约。

EvalScript 函数负责执行给定的脚本，验证其合法性，并根据执行结果返回相应的错误码。这个函数是比特币脚本验证的核心部分之一，主要逻辑是从待执行脚本中取出操作码并执行，直至取完、执行过程中遇到OP_RETURN、执行过程中VERIFY类验证失败、执行过程中遇到错误（例如操作数不足等）才会结束执行。我们的 AVM 就在这里加入了自己实现的一些OP。来达到对上下文的一个维护。

```
bool EvalScript(std::vector<valtype> &stack, const CScript &script, uint32_t flags, const BaseSignatureChecker &checker,
                ScriptExecutionMetrics &metrics, ScriptExecutionContextOpt const &context,
                ScriptStateContext &stateContext, ScriptError *serror, unsigned int *serror_op_num)

```
stack: 脚本执行时使用的堆栈。
script: 待执行的脚本。
flags: 脚本验证标志，用于启用特定的验证规则。
checker: 用于检查签名的基类。
metrics: 累积的脚本执行指标。
context: 脚本执行上下文选项。
stateContext: 脚本状态上下文。
serror: 可选的脚本错误指针，用于存储错误信息。
serror_op_num: 可选的脚本错误操作码，用于存储导致错误的具体操作码。

然后脚本执行起来，就进入到循环之中，各种case，处理不同的操作码。

OP_IF / OP_NOTIF:这些操作码处理条件语句。OP_IF 检查栈顶的值，如果为真则继续执行接下来的操作码；OP_NOTIF 则相反，如果栈顶值为假则继续执行。使用 vfExec 跟踪条件的执行状态。
检查条件值是否为布尔值，并将结果推入 vfExec 栈中。
OP_ELSE: 反转 vfExec 栈顶的布尔值，用于处理 else 分支。如果 vfExec 为空，则返回错误。
OP_ENDIF:结束一个条件语句块，将 vfExec 栈顶值弹出。如果 vfExec 为空，则返回错误。
OP_VERIFY: 验证栈顶值是否为真。如果为真，则弹出栈顶值；如果为假，则返回错误。
OP_RETURN: 终止脚本执行，表示脚本成功。如果栈不为空，则返回错误。
OP_TOALTSTACK / OP_FROMALTSTACK:
OP_TOALTSTACK 将栈顶值推入 altstack，并从主栈中弹出。
OP_FROMALTSTACK 将 altstack 栈顶值推入主栈，并从 altstack 中弹出。

本地内省操作码（无参数）:
OP_TXVERSION 获取交易版本号。
OP_TXINPUTCOUNT 获取交易输入计数。
OP_TXOUTPUTCOUNT 获取交易输出计数。
OP_TXLOCKTIME 获取交易锁定时间。
本地内省操作码（单参数）:
OP_OUTPOINTTXHASH 获取指定输入的交易哈希。
OP_OUTPOINTINDEX 获取指定输入的交易输出索引。
OP_INPUTBYTECODE 获取指定输入的脚本字节码。
OP_INPUTSEQUENCENUMBER 获取指定输入的序列号。
OP_INPUTWITNESSBYTECODE 获取指定输入的见证字节码。
OP_OUTPUTVALUE 获取指定输出的值。
OP_OUTPUTBYTECODE 获取指定输出的脚本字节码。

Atomicals 虚拟机操作码:
OP_NFT_PUT 处理 NFT 的放置操作。
OP_FT_COUNT 获取 FT 的计数，区分一般计数和传入计数。
OP_NFT_COUNT 获取 NFT 的计数，区分一般计数和传入计数。
OP_AUTH_INFO: 根据给定的输入索引，获取认证信息。如果索引无效或超出范围，会返回相应的错误。成功获取后，会将公钥信息压入栈中。
OP_FT_BALANCE_ADD: 增加特定代币的余额。检查金额是否有效，如果无效则返回错误。
OP_GETBLOCKINFO: 获取指定区块的信息，包括版本、前一个区块哈希、默克尔根、时间戳、难度等。根据提供的字段索引，将相应的信息推送到栈上。
OP_FT_BALANCE: 获取特定代币的余额。根据余额类型（当前余额或待确认余额），将余额推送到栈上。
OP_NFT_EXISTS: 检查特定的非同质代币 (NFT) 是否存在。根据类型（当前或待确认），将布尔值推送到栈上。
OP_FT_ITEM: 获取特定索引的代币 ID。根据类型（当前或待确认），将代币 ID 推送到栈上。
OP_NFT_ITEM: 获取特定索引的 NFT ID。根据类型（当前或待确认），将 NFT ID 推送到栈上。
OP_KV_EXISTS: 检查键值存储中是否存在指定的键值对。存在则推送 true 到栈上，否则推送 false。
OP_KV_GET: 获取键值存储中指定键的值，并将其推送到栈上。如果键不存在，则返回相应错误。
OP_KV_DELETE: 删除键值存储中的指定键值对。
OP_NFT_WITHDRAW: 从合约中提取指定 NFT。需要指定 NFT 的引用和输出索引。
OP_DECODEBLOCKINFO: 解码区块头信息。根据提供的区块头和字段索引，将相应的信息推送到栈上。
OP_HASH_FN: 执行哈希函数操作。根据提供的哈希函数索引（0 - SHA3-256, 1 - SHA-512, 2 - SHA-512/256, 3 - Eaglesong），对输入执行相应的哈希计算，并将结果推送到栈上。
OP_KV_PUT: 将键值对存储到状态中。首先检查键和值的大小是否超出限制，如果超出则返回相应错误。存储成功后，从栈中弹出相应的元素。
OP_FT_WITHDRAW: 从合约中提取特定的代币。需要提供代币引用、输出索引和提取的金额。检查提取金额是否有效，并执行提取操作。提取成功后，从栈中弹出相应的元素。

每个操作码都会检查上下文是否存在以及栈的大小是否足够进行操作。如果出现无效的操作码，则会返回 BAD_OPCODE 错误。同时，代码还包括一些异常处理和栈大小限制的检查。


随便看一段case代码：
```
case OP_FT_BALANCE: {
  if (vch1.size() != 36) {
      return set_error(serror, ScriptError::INVALID_ATOMICAL_REF_SIZE);
  }
  uint288 atomref(vch1);
  uint64_t balance = 0;
  CScriptNum sn(vch2, maxIntegerSize);
  auto balanceType = sn.getint();
  if (balanceType < 0 || balanceType > 1) {
      return set_error(serror, ScriptError::INVALID_AVM_FT_BALANCE_TYPE);
  }
  if (balanceType == 0) {
      balance = stateContext.contractFtBalance(atomref);
  } else {
      balance = stateContext.contractFtBalanceIncoming(atomref);
  }
  popstack(stack); // consume element
  popstack(stack); // consume element
  CScriptNum const bn(balance);
  stack.push_back(bn.getvch());
} break;
```

1. 检查引用大小: 首先，代码检查 vch1 的大小是否为36字节，这是确定代币引用的长度。如果不是，会返回 INVALID_ATOMICAL_REF_SIZE 错误。
2. 获取代币余额: 根据 vch1 中的代币引用创建一个 uint288 类型的 atomref，然后声明一个 balance 变量来存储余额。
3. 解析余额类型: 使用 vch2 中的数据创建一个 CScriptNum 对象 sn，并获取其整数值，表示余额类型。如果余额类型不在有效范围内（0或1），则返回 INVALID_AVM_FT_BALANCE_TYPE 错误。
4. 查询余额: 根据 balanceType 的值，调用 stateContext 对象的 contractFtBalance 或 contractFtBalanceIncoming 方法来获取相应的余额。这取决于余额类型，分别是合约内部余额或者进入的余额。
5. 栈操作: 一旦成功获取余额，代码从栈中弹出两个元素（即 vch1 和 vch2），然后将余额转换为 CScriptNum 类型的对象 bn，并将其字节序列推送回栈中。


具体的contractFtBalanceIncoming和contractFtBalance的方法在src/script/script_execution_context.cpp里面：
主要是对 ScriptStateContext 里面的内容进行维护：

```
uint64_t ScriptStateContext::contractFtBalance(const uint288 &ftId) {
    auto ftBalanceIt = _ftState.find(ftId.GetHex());
    if (ftBalanceIt == _ftState.end()) {
        // No balance found
        return 0;
    }
    return ftBalanceIt->template get<std::uint64_t>();
}

uint64_t ScriptStateContext::contractFtBalanceIncoming(const uint288 &ftId) {
    auto ftBalanceIt = _ftStateIncoming.find(ftId.GetHex());
    if (ftBalanceIt == _ftStateIncoming.end()) {
        // No balance found
        return 0;
    }
    return ftBalanceIt->template get<std::uint64_t>();
}
```

都用于从状态上下文中检索某个 ftId 的余额信息。它们分别从 _ftState 和 _ftStateIncoming 两个不同的映射中查找 ftId 对应的余额，并返回找到的余额值。如果没有找到，则返回 0。

## 总结一下

Atomicals Virtual Machine (AVM) 是一种用于执行特定操作码（opcode）的虚拟机，主要用于管理和操作非同质化代币（NFT）和可替代代币（FT）。以下是其主要功能和操作：

1. 基本结构和操作码
  AVM 包含多个操作码，这些操作码可以分为二元操作码和三元操作码两类。
  二元操作码：
  OP_KV_EXISTS: 检查键值对是否存在。
  OP_KV_GET: 获取键值对的值。
  OP_KV_DELETE: 删除键值对。
  OP_NFT_WITHDRAW: 提取 NFT。
  OP_HASH_FN: 执行特定的哈希函数。
  OP_GETBLOCKINFO: 获取区块信息。
  OP_DECODEBLOCKINFO: 解码区块信息。
  OP_FT_BALANCE: 获取 FT 的余额。
  OP_FT_ITEM: 获取 FT 项目。
  OP_NFT_ITEM: 获取 NFT 项目。
  OP_NFT_EXISTS: 检查 NFT 是否存在。
  OP_FT_BALANCE_ADD: 增加 FT 余额。
  OP_AUTH_INFO: 获取授权信息。
  三元操作码：
  OP_KV_PUT: 设置键值对。
  OP_FT_WITHDRAW: 提取 FT。

2. 主要操作码解释
  以下是一些关键操作码的详细解释：
  - OP_FT_BALANCE:
  获取 FT 的余额。操作码从堆栈中获取 atomref 和 balanceType，然后根据 balanceType 获取当前余额或即将到达的余额，并将结果压入堆栈。
  - OP_AUTH_INFO:
  获取授权信息。操作码从堆栈中获取授权输入索引和命名空间，检查索引是否有效，然后获取相应的授权信息。
  - OP_KV_PUT:
  设置键值对。操作码从堆栈中获取键、子键和值，然后将它们存储在状态上下文中。
  - OP_FT_WITHDRAW:
  提取 FT。操作码从堆栈中获取 atomref、输出索引和提取金额，然后检查这些参数是否有效，并执行提取操作。

3. 状态上下文管理
  AVM 使用 ScriptStateContext 来管理状态上下文，包含以下关键函数：
  contractFtBalance: 获取 FT 的当前余额。
  contractFtBalanceIncoming: 获取 FT 的即将到达的余额。

4. 堆栈操作
AVM 使用堆栈来处理操作码的输入和输出。大部分操作码从堆栈中弹出所需的参数，并在执行完成后将结果压入堆栈。

5. 错误处理
AVM 包含详细的错误处理机制，确保在操作码执行过程中遇到错误时能够及时返回相应的错误代码。

总结
AVM 是一个高度模块化和可扩展的虚拟机，专为管理和操作 NFT 和 FT 设计。通过一组精确定义的操作码，AVM 能够执行各种复杂的操作，确保在区块链环境中的高效和安全运行。


相关资料：
https://doxygen.bitcoincore.org/class_c_script.html
https://blog.csdn.net/u013434801/article/details/120636272
https://startbitcoin.org/5523/
https://startbitcoin.org/5508/
https://scrypt.io/
