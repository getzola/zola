
+++
title = "BelResume"
description = "A beautiful, modern, and minimal single-page resume site"
template = "theme.html"
date = 2025-04-30T17:15:25+05:30

[taxonomies]
theme-tags = ['minimal', 'resume']

[extra]
created = 2025-04-30T17:15:25+05:30
updated = 2025-04-30T17:15:25+05:30
repository = "https://github.com/cx48/BelResume.git"
homepage = "https://github.com/cx48/BelResume"
minimum_version = "0.20.0"
license = "MIT"
demo = "https://cx48.github.io/BelResume/"

[extra.author]
name = "cx48"
homepage = "https://cx48.dev"
+++        

# BelResumé

***A beautiful, modern, and minimal single-page resume site***

Powered by [Zola](https://getzola.org/). Styled with Tailwind CSS & Font Awesome

> “Bel” in French means beautiful — this is your beautiful resumé!

## Preview

[BelResumé](https://cx48.github.io/BelResume/) can be deployed for free using GitHub Pages or Vercel

#### Light Mode

![Light Mode](https://raw.githubusercontent.com/cx48/BelResume/refs/heads/main/static/images/light.png)

#### Dark Mode

![Dark Mode](https://raw.githubusercontent.com/cx48/BelResume/refs/heads/main/static/images/dark.png)

#### PageSpeed Insights

![PageSpeed](https://raw.githubusercontent.com/cx48/BelResume/refs/heads/main/static/images/pagespeed.png)

## Project Structure

- **config.toml**: Site metadata  
- **static/**: `css/style.css`, `js/script.js`  
- **templates/index.html**: Main layout (calls all partials)  
- **templates/partials/**: All resume sections are modular
  - `header.html`: Name, job title, contact links
  - `experience.html`: Work history (companies, roles, achievements)
  - `education.html`: Schools, degrees, specializations
  - `projects.html`: Key project summaries with tags
  - `skills.html`: Visual skill bars (adjust widths)
  - `certifications.html`: Cert title + authority + year
  - `languages.html`: Language proficiency (bars & labels)
  - `awards.html`: Award names + year + issuer

## Quick Start

1. **Install Zola**: [https://getzola.org/documentation/getting-started/installation/](https://getzola.org/documentation/getting-started/installation/) 

2. **Clone repository**:
   ```bash
   git clone https://github.com/cx48/BelResume
   cd BelResume
   ```

3. **Serve locally**:
   ```bash
   zola serve
   ```

   After making necessary changes to HTML files present under `partials/` visit [http://127.0.0.1:1111](http://127.0.0.1:1111)

4. **Build static site**:
   ```bash
   zola build
   ```
   All files output to `/public`

## Deployment Guide

> Deploy to GitHub Pages

1. Run Zola build:
   ```bash
   zola build
   ```

2. Commit and push the contents of the `public/` folder to your `gh-pages` branch

3. In GitHub repo settings, enable Pages from the `/public` folder or `gh-pages` branch

4. Your site will be live at `https://yourusername.github.io/BelResume/`

> Deploy to Vercel

1. Login to [Vercel](https://vercel.com) and import your GitHub repo

2. Set **Build Command** to:
   ```bash
   zola build
   ```

3. Set **Output Directory** to:
   ```bash
   public
   ```

4. Set **Framework Preset** to `Other`

5. Click **Deploy**

Zola will build and serve from the `public/` folder automatically on every push

## Customization Guide

To update your resume, simply open the required HTML file under `partials/` and modify it as per your requirement

### 1. **config.toml**  
Update site-wide metadata:
```toml
title = "John Doe"
description = "Senior Software Engineer"
base_url = "https://yourdomain.com"
```

### 2. **partials/header.html**
Edit your name, role, email, phone, location, LinkedIn link. Example:
```html
<h1 class="text-3xl font-bold">John Doe</h1>
<h2 class="text-xl text-[var(--primary)] font-semibold">Senior Software Engineer</h2>
```

### 3. **partials/experience.html**
Replace job titles, companies, durations, and bullet points.
```html
<h3 class="font-semibold">Senior Developer</h3>
<span>2020–Present</span>
<ul>
  <li>Improved system performance by 30%</li>
</ul>
```

### 4. **partials/education.html**
List your degrees, schools, GPA or honors:
```html
<h3>Master of CS</h3>
<p>Stanford University — GPA: 3.9</p>
```

### 5. **partials/projects.html**
List project names, stacks used, short descriptions.
```html
<h3>E-commerce Platform</h3>
<span class="tag">React</span> <span class="tag">Node.js</span>
```

### 6. **partials/skills.html**
Update skill bars by adjusting the width in `style="width: 90%"`.
```html
<h3>JavaScript</h3>
<div class="progress" style="width: 95%"></div>
```

### 7. **partials/certifications.html**
List certificates with name, organization, and date:
```html
<h3>AWS Certified Architect</h3>
<p>Amazon – 2022</p>
```

### 8. **partials/languages.html**
Change language names, levels, and progress widths:
```html
<span>English</span> <span>Native</span>
<div style="width: 100%"></div>
```

### 9. **partials/awards.html**
Highlight awards with name and source:
```html
<h3>Best Hackathon Project</h3>
<p>DefCon 2025</p>
```

## Get in Touch

Have feedback, questions, or just want to say hello?  
Feel free to [open an issue](https://github.com/cx48/BelResume/issues) or reach out directly:

> Check my GitHub profile for contact links

        