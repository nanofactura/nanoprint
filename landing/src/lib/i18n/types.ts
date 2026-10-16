import type { downloads } from '$lib/site';

export interface Faq {
  q: string;
  a: string;
}

export interface Feature {
  title: string;
  text: string;
  points: string[];
}

export interface Step {
  title: string;
  text: string;
}

export interface PlatformInfo {
  name: string;
  requirements: string;
  files: { label: string; download: keyof typeof downloads }[];
  steps: string[];
}

export interface Library {
  name: string;
  href: string;
  license: string;
  role: string;
  note: string;
}

/** Everything the site says, in one language. Both languages must define every entry. */
export interface Messages {
  meta: {
    /** Open Graph locale, such as `es_AR`. */
    locale: string;
  };
  nav: {
    features: string;
    download: string;
    developers: string;
    faq: string;
    github: string;
    /** The other language's name, in that language: the switch reads in the visitor's tongue. */
    otherLanguage: string;
    languageLabel: string;
    skip: string;
    menu: string;
  };
  download: {
    generic: string;
    windows: string;
    mac: string;
    linux: string;
    otherPlatforms: string;
    previousVersions: string;
    desktopOnly: string;
    starting: string;
    nextSteps: string;
    retry: string;
  };
  home: {
    title: string;
    description: string;
    hero: {
      title: string;
      lead: string;
      platforms: string;
      free: string;
      openSource: string;
      videoLabel: string;
    };
    features: { title: string; lead: string; items: [Feature, Feature, Feature] };
    how: { title: string; items: Step[] };
    integrate: { title: string; text: string; points: string[]; button: string; code: string };
    printers: {
      title: string;
      text: string;
      driversTitle: string;
      drivers: { os: string; text: string }[];
      note: string;
    };
    downloads: {
      title: string;
      lead: string;
      requirements: string;
      install: string;
      windows: PlatformInfo;
      mac: PlatformInfo;
      linux: PlatformInfo;
      afterTitle: string;
      afterText: string;
      previous: string;
      verifyTitle: string;
      verifyText: string;
    };
    openSource: { title: string; text: string; button: string };
    signing: { title: string; items: string[] };
    faq: { title: string; items: Faq[] };
  };
  developers: {
    title: string;
    description: string;
    hero: { eyebrow: string; title: string; lead: string; guide: string; protocol: string };
    libraries: { title: string; lead: string; items: Library[]; license: string };
    example: { title: string; lead: string; code: string; note: string };
    identity: { title: string; text: string; code: string };
    wake: { title: string; text: string; code: string; note: string };
    demo: {
      title: string;
      lead: string;
      steps: string[];
      connect: string;
      wake: string;
      wakeFailed: string;
      connecting: string;
      connected: string;
      disconnect: string;
      printer: string;
      paper: string;
      qr: string;
      print: string;
      printing: string;
      printed: string;
      noPrinters: string;
      notRunning: string;
      blocked: string;
      failed: string;
      download: string;
      ticket: {
        title: string;
        items: [string, number, number][];
        total: string;
        thanks: string;
        jobName: string;
      };
    };
    adopt: { title: string; lead: string; items: string[]; issues: string };
    requirements: { title: string; items: string[] };
    more: string;
  };
  footer: {
    productOf: string;
    links: {
      github: string;
      license: string;
      security: string;
      licenses: string;
      developers: string;
      signing: string;
    };
    legal: string[];
    privacy: string;
  };
  code: { copy: string; copied: string };
}
