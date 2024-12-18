import type { SidebarsConfig } from "@docusaurus/plugin-content-docs";

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */
const sidebars: SidebarsConfig = {
  // By default, Docusaurus generates a sidebar from the docs folder structure
  tutorialSidebar: [
    { type: "autogenerated", dirName: "." },
    {
      type: "link",
      label: "Whitepaper",
      href: "https://ts.docsend.com/view/5xyq5v7burxnmkib",
    },
  ],

  // But you can create a sidebar manually
  /*
  tutorialSidebar: [
    {
      type: "category",
      label: "About",
      collapsed: false,
      items: [
        "INTRODUCTION",
        "KEY_TERMS",
        "WHY_QUIBLE",
        {
          type: "category",
          collapsed: false,
          label: "QBL Token",
          items: [
            {
              type: "doc",
              label: "Overview",
              id: "QBL_TOKEN_OVERVIEW",
            },
            {
              type: "doc",
              label: "Economy & Costs",
              id: "QBL_TOKEN_ECONOMY",
            },
          ],
        },
        "USE_CASES",
        "COMMUNITY",
        "ROADMAP",
      ],
    },
    {
      type: "category",
      collapsed: false,
      label: "Architecture",
      items: [
        {
          type: "doc",
          label: "Overview",
          id: "ARCHITECTURE",
        },
        "QUIBLE_IDENTITY_MANAGEMENT",
        "EVM_IDENTITY_MANAGEMENT",
        "CERTIFICATES",
        "SECURITY",
      ],
    },
    {
      type: "category",
      collapsed: false,
      label: "Operator",
      items: [
        {
          type: "doc",
          label: "Node Tiers",
          id: "NODE_TIERS",
        },
        {
          type: "doc",
          label: "Requirements",
          id: "OPERATOR_REQUIREMENTS",
        },
        {
          type: "doc",
          label: "Installation",
          id: "OPERATOR_INSTALLATION",
        },
      ],
    },
    {
      type: "category",
      collapsed: false,
      label: "Usage Guides",
      items: [
        {
          type: "doc",
          label: "API Reference",
          id: "API",
        },
        {
          type: "doc",
          label: "TypeScript SDK",
          id: "SDK",
        },
        {
          type: "doc",
          label: "Relay NFTs from EVM",
          id: "BRIDGING_GUIDE",
        },
      ],
    },
    {
      type: 'doc',
      label: 'Frequently Asked Questions',
      id: 'FAQ'
    },
    {
      type: 'link',
      label: 'Whitepaper',
      href: 'https://ts.docsend.com/view/5xyq5v7burxnmkib'
    }
  ],
  */
};

export default sidebars;
