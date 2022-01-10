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
```bash
npm init
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

5. Build your zola app:
```bash
zola build
```

6. Deploy!
```bash
0 deploy
```
