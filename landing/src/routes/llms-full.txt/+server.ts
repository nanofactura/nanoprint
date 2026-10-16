import { messages, pages, type Lang } from '$lib/i18n';
import { RELEASES_URL, REPO_URL, SITE_URL } from '$lib/site';

export const prerender = true;
export const trailingSlash = 'never';

/** Backticks mark code in the catalogs; they read fine in Markdown as they are. */
function section(lang: Lang): string {
  const t = messages(lang);
  const { home, developers: dev } = t;
  const list = (items: string[]) => items.map((item) => `- ${item}`).join('\n');
  const platforms = [home.downloads.windows, home.downloads.mac, home.downloads.linux];
  return `# ${home.title}

${home.description}

URL: ${SITE_URL}${pages.home[lang]}

## ${home.features.title}

${home.features.items.map((item) => `### ${item.title}\n\n${item.text}\n\n${list(item.points)}`).join('\n\n')}

## ${home.how.title}

${home.how.items.map((step, index) => `${String(index + 1)}. ${step.title}: ${step.text}`).join('\n')}

## ${home.printers.title}

${home.printers.text}

### ${home.printers.driversTitle}

${home.printers.drivers.map((driver) => `- ${driver.os}: ${driver.text}`).join('\n')}

${home.printers.note}

## ${home.downloads.title}

${platforms.map((platform) => `### ${platform.name}\n\n${home.downloads.requirements}: ${platform.requirements}\n\n${platform.steps.map((step, index) => `${String(index + 1)}. ${step}`).join('\n')}`).join('\n\n')}

${home.downloads.afterText}

${RELEASES_URL}

## ${home.faq.title}

${home.faq.items.map((item) => `### ${item.q}\n\n${item.a}`).join('\n\n')}

## ${dev.title}

URL: ${SITE_URL}${pages.developers[lang]}

${dev.hero.lead}

### ${dev.libraries.title}

${dev.libraries.items.map((library) => `- ${library.name} (${library.license}): ${library.role} ${library.note} ${library.href}`).join('\n')}

### ${dev.example.title}

${dev.example.lead}

\`\`\`js
${dev.example.code}
\`\`\`

${dev.example.note}

### ${dev.identity.title}

${dev.identity.text}

\`\`\`js
${dev.identity.code}
\`\`\`

### ${dev.wake.title}

${dev.wake.text}

\`\`\`js
${dev.wake.code}
\`\`\`

${dev.wake.note}

### ${dev.adopt.title}

${list(dev.adopt.items)}

### ${dev.requirements.title}

${list(dev.requirements.items)}
`;
}

/** The whole site as plain text for AI assistants (https://llmstxt.org), in both languages. */
export function GET(): Response {
  const text = `${section('en')}
---

${section('es')}
---

Source code and documentation: ${REPO_URL}
`;
  return new Response(text, { headers: { 'content-type': 'text/plain; charset=utf-8' } });
}
