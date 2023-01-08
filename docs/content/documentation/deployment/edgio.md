+++
title = "Edgio"
weight = 50
+++

If you don't have an account with Edgio, you can sign up [here](https://app.layer0.co/signup).

## Manual deploys

For a command-line manual deploy, follow these steps:

1. Install the Edgio CLI: 

```bash
npm i -g @edgio/cli
```

2. Create a package.json at the root of your project with the following:

```bash
npm init -y
```

3. Initialize your project with:

```bash
edgio init
```

4. Update routes.js at the root of your project to the following:

```js
// This file was added by edgio init.
// You should commit this file to source control.

import { Router } from '@edgio/core/router'

export default new Router().static('public')
```

5. Build your zola app:

```bash
zola build
```

6. Deploy!

```bash
edgio deploy
```
