const baseLocale = "en";
const locales = (
  /** @type {const} */
  ["en", "es"]
);
const localStorageKey = "PARAGLIDE_LOCALE";
const strategy = [
  "localStorage",
  "preferredLanguage",
  "baseLocale"
];
globalThis.__paraglide = /** @type {any} */
globalThis.__paraglide ?? {};
globalThis.__paraglide.ssr = /** @type {any} */
globalThis.__paraglide.ssr ?? {};
let localeInitiallySet = false;
let getLocale = () => {
  let strategyToUse = strategy;
  const resolved = resolveLocaleWithStrategies(strategyToUse);
  if (resolved) {
    if (!localeInitiallySet) {
      localeInitiallySet = true;
      setLocale(resolved, { reload: false });
    }
    return resolved;
  }
  throw new Error("No locale found. Read the docs https://paraglidejs.com/errors#no-locale-found");
};
function resolveLocaleWithStrategies(strategyToUse, urlForUrlStrategy) {
  let locale;
  for (const strat of strategyToUse) {
    if (strat === "baseLocale") {
      locale = baseLocale;
    } else if (isCustomStrategy(strat) && customClientStrategies.has(strat)) {
      const handler = customClientStrategies.get(strat);
      if (handler) {
        const result = handler.getLocale();
        if (result instanceof Promise) {
          continue;
        }
        if (result !== void 0) {
          return assertIsLocale(result);
        }
      }
    }
    const matchedLocale = toLocale(locale);
    if (matchedLocale) {
      return matchedLocale;
    }
  }
  return void 0;
}
let setLocale = (newLocale, options) => {
  ({
    ...options
  });
  let currentLocale;
  try {
    currentLocale = getLocale();
  } catch {
  }
  const customSetLocalePromises = [];
  let strategyToUse = strategy;
  for (const strat of strategyToUse) {
    if (strat === "baseLocale") {
      continue;
    } else if (strat === "localStorage" && typeof window !== "undefined") {
      localStorage.setItem(localStorageKey, newLocale);
    } else if (isCustomStrategy(strat) && customClientStrategies.has(strat)) {
      const handler = customClientStrategies.get(strat);
      if (handler) {
        let result = handler.setLocale(newLocale);
        if (result instanceof Promise) {
          result = result.catch((error) => {
            throw new Error(`Custom strategy "${strat}" setLocale failed.`, {
              cause: error
            });
          });
          customSetLocalePromises.push(result);
        }
      }
    }
  }
  if (customSetLocalePromises.length) {
    return Promise.all(customSetLocalePromises).then(() => {
    });
  }
  return;
};
function toLocale(value) {
  if (typeof value !== "string") {
    return void 0;
  }
  const lowerValue = value.toLowerCase();
  for (const locale of locales) {
    if (locale.toLowerCase() === lowerValue) {
      return locale;
    }
  }
  return void 0;
}
function assertIsLocale(input) {
  const locale = toLocale(input);
  if (locale)
    return locale;
  throw new Error(`Invalid locale: ${input}. Expected one of: ${locales.join(", ")}`);
}
const customClientStrategies = /* @__PURE__ */ new Map();
function isCustomStrategy(strategy2) {
  return typeof strategy2 === "string" && /^custom-[A-Za-z0-9_-]+$/.test(strategy2);
}
export {
  getLocale as g
};
