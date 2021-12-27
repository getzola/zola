+++
title = "Layer0"
weight = 50
+++

If you don't have an account with Layer0, you can sign up [here](https://app.layer0.co/signup).

## Manual deploys

For a command-line manual deploy, follow these steps:
1. Install the Layer0 CLI: 
```bash
npm i -g @layer0/cli
```

2. Create a package.json at the root of your project with the following:
```js
{
  "name": "zola",
  "version": "1.0.0",
  "scripts": {
    "build": "zola build",
    "layer0:dev": "0 dev",
    "layer0:buid": "0 build",
    "layer0:deploy": "0 deploy"
  },
  "dependencies": {},
  "devDependencies": {}
}
```

3. Initialize your project with:
```bash
0 init
```

4. Update routes.js at the root of your project to the following:
```js
// This file was added by layer0 init.
// You should commit this file to source control.

import { Router } from '@layer0/core/router'

export default new Router().static('public', ({ cache }) => {
  cache({
    edge: {
      maxAgeSeconds: 60 * 60 * 60 * 365,
      forcePrivateCaching: true,
    },
    browser: {
      maxAgeSeconds: 0,
      serviceWorkerSeconds: 60 * 60 * 24,
    },
  })
})
```

5. Deploy with:
```bash
0 deploy
```
