# @heima-network/chaindata

This library contains information about the available networks at Heima, including its testnets.

## Installation

1. Install from NPM

   ```
   npm install @heima-network/chaindata
   ```

2. Explore

   ```ts
   import { all, byId } from '@heima-network/chaindata`;

   console.log(all);

   console.log(`Heima's production RPC URL is: ${byId['heima-prod'].rpcs[0].url}`);
   ```
