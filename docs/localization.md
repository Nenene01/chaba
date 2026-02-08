# Localization Guide

## Adding a New Language

We welcome translations to make Chaba accessible to developers worldwide!

### Structure

```
docs/
├── en/                 # English documentation
├── ja/                 # Japanese documentation
├── zh/                 # Chinese documentation (help wanted!)
├── ko/                 # Korean documentation (help wanted!)
└── [language-code]/    # Your language
```

### Files to Translate

1. **README.md** (root) - Main project page
2. **CONTRIBUTING.md** - Contribution guidelines
3. **docs/[lang]/getting-started.md** - Getting started guide
4. **docs/[lang]/configuration.md** - Configuration reference
5. **docs/[lang]/architecture.md** - Architecture overview

### Translation Workflow

1. **Create a directory** for your language:
   ```bash
   mkdir docs/[language-code]
   ```

2. **Copy English templates**:
   ```bash
   cp docs/en/*.md docs/[language-code]/
   ```

3. **Translate the content**:
   - Keep code examples unchanged
   - Translate comments in code if helpful
   - Maintain the same structure

4. **Update README.md**:
   - Add a language switcher link at the top
   - Follow the existing format

5. **Submit a Pull Request**:
   - Title: `docs: add [Language] translation`
   - Include which files you translated

### Language Codes

Use ISO 639-1 codes:
- `en` - English
- `ja` - Japanese (日本語)
- `zh` - Chinese (中文)
- `ko` - Korean (한국어)
- `es` - Spanish (Español)
- `fr` - French (Français)
- `de` - German (Deutsch)
- `pt` - Portuguese (Português)
- `ru` - Russian (Русский)

### Translation Tips

- **Be natural**: Translate meaning, not word-by-word
- **Keep technical terms**: Some terms like "git worktree" may not need translation
- **Use local conventions**: Follow your language's documentation standards
- **Ask for help**: Open a Discussion if you're unsure

### Maintaining Translations

When the English docs are updated:
1. We'll create an issue for each language
2. Translators can update their language version
3. Mark outdated translations with a notice

### Questions?

Open a [Discussion](https://github.com/Nenene01/chaba/discussions) or ask in your PR!

---

Thank you for helping make Chaba accessible to everyone! 🍵🌏
