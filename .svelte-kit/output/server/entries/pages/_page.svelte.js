import { e as escape_html, a4 as attr, a5 as attr_class, a6 as bind_props, a7 as ensure_array_like } from "../../chunks/index.js";
import "@tauri-apps/api/core";
import "@tauri-apps/api/event";
import "@tauri-apps/plugin-dialog";
import { g as getLocale } from "../../chunks/runtime.js";
const en_app_title = (
  /** @type {(inputs: App_TitleInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Open Crawler`
    );
  }
);
const es_app_title = (
  /** @type {(inputs: App_TitleInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Open Crawler`
    );
  }
);
const app_title = (
  /** @type {((inputs?: App_TitleInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<App_TitleInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_app_title();
    return en_app_title();
  })
);
const en_app_select_project = (
  /** @type {(inputs: App_Select_ProjectInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Select or create a project`
    );
  }
);
const es_app_select_project = (
  /** @type {(inputs: App_Select_ProjectInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Selecciona o crea un proyecto`
    );
  }
);
const app_select_project = (
  /** @type {((inputs?: App_Select_ProjectInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<App_Select_ProjectInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_app_select_project();
    return en_app_select_project();
  })
);
const en_app_select_project_hint = (
  /** @type {(inputs: App_Select_Project_HintInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Choose a project from the sidebar or create a new one to start crawling.`
    );
  }
);
const es_app_select_project_hint = (
  /** @type {(inputs: App_Select_Project_HintInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Elige un proyecto de la barra lateral o crea uno nuevo para comenzar a rastrear.`
    );
  }
);
const app_select_project_hint = (
  /** @type {((inputs?: App_Select_Project_HintInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<App_Select_Project_HintInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_app_select_project_hint();
    return en_app_select_project_hint();
  })
);
const en_sidebar_new_project_placeholder = (
  /** @type {(inputs: Sidebar_New_Project_PlaceholderInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `New project name...`
    );
  }
);
const es_sidebar_new_project_placeholder = (
  /** @type {(inputs: Sidebar_New_Project_PlaceholderInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Nombre del nuevo proyecto...`
    );
  }
);
const sidebar_new_project_placeholder = (
  /** @type {((inputs?: Sidebar_New_Project_PlaceholderInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Sidebar_New_Project_PlaceholderInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_sidebar_new_project_placeholder();
    return en_sidebar_new_project_placeholder();
  })
);
const en_sidebar_no_projects = (
  /** @type {(inputs: Sidebar_No_ProjectsInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `No projects yet. Create one above.`
    );
  }
);
const es_sidebar_no_projects = (
  /** @type {(inputs: Sidebar_No_ProjectsInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Aún no hay proyectos. Crea uno arriba.`
    );
  }
);
const sidebar_no_projects = (
  /** @type {((inputs?: Sidebar_No_ProjectsInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Sidebar_No_ProjectsInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_sidebar_no_projects();
    return en_sidebar_no_projects();
  })
);
const en_sidebar_rename = (
  /** @type {(inputs: Sidebar_RenameInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Rename`
    );
  }
);
const es_sidebar_rename = (
  /** @type {(inputs: Sidebar_RenameInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Renombrar`
    );
  }
);
const sidebar_rename = (
  /** @type {((inputs?: Sidebar_RenameInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Sidebar_RenameInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_sidebar_rename();
    return en_sidebar_rename();
  })
);
const en_sidebar_delete = (
  /** @type {(inputs: Sidebar_DeleteInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Delete`
    );
  }
);
const es_sidebar_delete = (
  /** @type {(inputs: Sidebar_DeleteInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Eliminar`
    );
  }
);
const sidebar_delete = (
  /** @type {((inputs?: Sidebar_DeleteInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Sidebar_DeleteInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_sidebar_delete();
    return en_sidebar_delete();
  })
);
const en_config_max_depth = (
  /** @type {(inputs: Config_Max_DepthInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Max Depth`
    );
  }
);
const es_config_max_depth = (
  /** @type {(inputs: Config_Max_DepthInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Profundidad Máxima`
    );
  }
);
const config_max_depth = (
  /** @type {((inputs?: Config_Max_DepthInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Config_Max_DepthInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_config_max_depth();
    return en_config_max_depth();
  })
);
const en_config_time_limit = (
  /** @type {(inputs: Config_Time_LimitInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Time Limit (seconds)`
    );
  }
);
const es_config_time_limit = (
  /** @type {(inputs: Config_Time_LimitInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Límite de Tiempo (segundos)`
    );
  }
);
const config_time_limit = (
  /** @type {((inputs?: Config_Time_LimitInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Config_Time_LimitInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_config_time_limit();
    return en_config_time_limit();
  })
);
const en_config_respect_robots = (
  /** @type {(inputs: Config_Respect_RobotsInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Respect robots.txt`
    );
  }
);
const es_config_respect_robots = (
  /** @type {(inputs: Config_Respect_RobotsInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Respetar robots.txt`
    );
  }
);
const config_respect_robots = (
  /** @type {((inputs?: Config_Respect_RobotsInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Config_Respect_RobotsInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_config_respect_robots();
    return en_config_respect_robots();
  })
);
const en_config_check_sitemap = (
  /** @type {(inputs: Config_Check_SitemapInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Check sitemap.xml`
    );
  }
);
const es_config_check_sitemap = (
  /** @type {(inputs: Config_Check_SitemapInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Verificar sitemap.xml`
    );
  }
);
const config_check_sitemap = (
  /** @type {((inputs?: Config_Check_SitemapInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Config_Check_SitemapInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_config_check_sitemap();
    return en_config_check_sitemap();
  })
);
const en_config_check_semantics = (
  /** @type {(inputs: Config_Check_SemanticsInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Check semantic HTML`
    );
  }
);
const es_config_check_semantics = (
  /** @type {(inputs: Config_Check_SemanticsInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Verificar HTML semántico`
    );
  }
);
const config_check_semantics = (
  /** @type {((inputs?: Config_Check_SemanticsInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Config_Check_SemanticsInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_config_check_semantics();
    return en_config_check_semantics();
  })
);
const en_detail_back = (
  /** @type {(inputs: Detail_BackInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `← Results`
    );
  }
);
const es_detail_back = (
  /** @type {(inputs: Detail_BackInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `← Resultados`
    );
  }
);
const detail_back = (
  /** @type {((inputs?: Detail_BackInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Detail_BackInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_detail_back();
    return en_detail_back();
  })
);
const en_detail_loading = (
  /** @type {(inputs: Detail_LoadingInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Loading page details...`
    );
  }
);
const es_detail_loading = (
  /** @type {(inputs: Detail_LoadingInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Cargando detalles de la página...`
    );
  }
);
const detail_loading = (
  /** @type {((inputs?: Detail_LoadingInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Detail_LoadingInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_detail_loading();
    return en_detail_loading();
  })
);
const en_detail_overview = (
  /** @type {(inputs: Detail_OverviewInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Overview`
    );
  }
);
const es_detail_overview = (
  /** @type {(inputs: Detail_OverviewInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Resumen`
    );
  }
);
const detail_overview = (
  /** @type {((inputs?: Detail_OverviewInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Detail_OverviewInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_detail_overview();
    return en_detail_overview();
  })
);
const en_detail_links = (
  /** @type {(inputs: Detail_LinksInputs) => LocalizedString} */
  (i) => {
    return (
      /** @type {LocalizedString} */
      `Links (${i?.count})`
    );
  }
);
const es_detail_links = (
  /** @type {(inputs: Detail_LinksInputs) => LocalizedString} */
  (i) => {
    return (
      /** @type {LocalizedString} */
      `Enlaces (${i?.count})`
    );
  }
);
const detail_links = (
  /** @type {((inputs: Detail_LinksInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Detail_LinksInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_detail_links(inputs);
    return en_detail_links(inputs);
  })
);
const en_language_label = (
  /** @type {(inputs: Language_LabelInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Language`
    );
  }
);
const es_language_label = (
  /** @type {(inputs: Language_LabelInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Idioma`
    );
  }
);
const language_label = (
  /** @type {((inputs?: Language_LabelInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Language_LabelInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_language_label();
    return en_language_label();
  })
);
const en_language_en = (
  /** @type {(inputs: Language_EnInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `English`
    );
  }
);
const es_language_en = (
  /** @type {(inputs: Language_EnInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `English`
    );
  }
);
const language_en = (
  /** @type {((inputs?: Language_EnInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Language_EnInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_language_en();
    return en_language_en();
  })
);
const en_language_es = (
  /** @type {(inputs: Language_EsInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Español`
    );
  }
);
const es_language_es = (
  /** @type {(inputs: Language_EsInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Español`
    );
  }
);
const language_es = (
  /** @type {((inputs?: Language_EsInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Language_EsInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_language_es();
    return en_language_es();
  })
);
const en_settings_title = (
  /** @type {(inputs: Settings_TitleInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Settings`
    );
  }
);
const es_settings_title = (
  /** @type {(inputs: Settings_TitleInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Configuración`
    );
  }
);
const settings_title = (
  /** @type {((inputs?: Settings_TitleInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Settings_TitleInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_settings_title();
    return en_settings_title();
  })
);
const en_settings_page_size = (
  /** @type {(inputs: Settings_Page_SizeInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Results per page`
    );
  }
);
const es_settings_page_size = (
  /** @type {(inputs: Settings_Page_SizeInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Resultados por página`
    );
  }
);
const settings_page_size = (
  /** @type {((inputs?: Settings_Page_SizeInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Settings_Page_SizeInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_settings_page_size();
    return en_settings_page_size();
  })
);
const en_settings_default_config = (
  /** @type {(inputs: Settings_Default_ConfigInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Default Crawl Config`
    );
  }
);
const es_settings_default_config = (
  /** @type {(inputs: Settings_Default_ConfigInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Configuración por Defecto`
    );
  }
);
const settings_default_config = (
  /** @type {((inputs?: Settings_Default_ConfigInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Settings_Default_ConfigInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_settings_default_config();
    return en_settings_default_config();
  })
);
const en_settings_cancel = (
  /** @type {(inputs: Settings_CancelInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Cancel`
    );
  }
);
const es_settings_cancel = (
  /** @type {(inputs: Settings_CancelInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Cancelar`
    );
  }
);
const settings_cancel = (
  /** @type {((inputs?: Settings_CancelInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Settings_CancelInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_settings_cancel();
    return en_settings_cancel();
  })
);
const en_settings_save = (
  /** @type {(inputs: Settings_SaveInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Save`
    );
  }
);
const es_settings_save = (
  /** @type {(inputs: Settings_SaveInputs) => LocalizedString} */
  () => {
    return (
      /** @type {LocalizedString} */
      `Guardar`
    );
  }
);
const settings_save = (
  /** @type {((inputs?: Settings_SaveInputs, options?: { locale?: "en" | "es" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Settings_SaveInputs, { locale?: "en" | "es" }, {}>} */
  ((inputs = {}, options = {}) => {
    const locale = options.locale ?? getLocale();
    if (locale === "es") return es_settings_save();
    return en_settings_save();
  })
);
function PageDetailPanel($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { pageId = "", onClose } = $$props;
    let detail = null;
    let links = [];
    let activeTab = "overview";
    if (pageId) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div class="fullpage svelte-2bg1ls"><header class="fullpage-header svelte-2bg1ls"><div class="header-left svelte-2bg1ls"><button class="btn-back svelte-2bg1ls" aria-label="Back to results">${escape_html(detail_back())}</button> <h3 class="page-url svelte-2bg1ls"${attr("title", detail?.url)}>${escape_html(detail_loading())}</h3></div> <div class="header-right svelte-2bg1ls">`);
      {
        $$renderer2.push("<!--[-1-->");
      }
      $$renderer2.push(`<!--]--></div></header> <div class="tab-bar svelte-2bg1ls"><button${attr_class("tab svelte-2bg1ls", void 0, { "active": activeTab === "overview" })}>${escape_html(detail_overview())}</button> <button${attr_class("tab svelte-2bg1ls", void 0, { "active": activeTab === "links" })}>${escape_html(detail_links({ count: links.length.toString() }))}</button></div> `);
      {
        $$renderer2.push("<!--[-1-->");
      }
      $$renderer2.push(`<!--]--></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]-->`);
    bind_props($$props, { pageId });
  });
}
function SettingsModal($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { open = false, onsave } = $$props;
    let language = getLocale();
    let pageSize = "50";
    let maxDepth = "10";
    let respectRobots = true;
    let checkSitemap = true;
    let checkSemantics = true;
    let maxCrawlTime = 3600;
    let saving = false;
    if (open) {
      $$renderer2.push("<!--[0-->");
      $$renderer2.push(`<div class="modal-overlay svelte-1wk5kq2" role="presentation"><div class="modal svelte-1wk5kq2" role="dialog" tabindex="-1" aria-label="Settings"><div class="modal-header svelte-1wk5kq2"><h2 class="svelte-1wk5kq2">${escape_html(settings_title())}</h2> <button class="btn-close svelte-1wk5kq2" aria-label="Close">✕</button></div> <div class="modal-body svelte-1wk5kq2"><div class="setting-group svelte-1wk5kq2"><label for="lang" class="svelte-1wk5kq2">${escape_html(language_label())}</label> `);
      $$renderer2.select(
        { id: "lang", value: language, class: "" },
        ($$renderer3) => {
          $$renderer3.option({ value: "en" }, ($$renderer4) => {
            $$renderer4.push(`${escape_html(language_en())}`);
          });
          $$renderer3.option({ value: "es" }, ($$renderer4) => {
            $$renderer4.push(`${escape_html(language_es())}`);
          });
        },
        "svelte-1wk5kq2"
      );
      $$renderer2.push(`</div> <div class="setting-divider svelte-1wk5kq2"></div> <div class="setting-group svelte-1wk5kq2"><label for="page-size" class="svelte-1wk5kq2">${escape_html(settings_page_size())}</label> `);
      $$renderer2.select(
        { id: "page-size", value: pageSize, class: "" },
        ($$renderer3) => {
          $$renderer3.option({ value: "25" }, ($$renderer4) => {
            $$renderer4.push(`25`);
          });
          $$renderer3.option({ value: "50" }, ($$renderer4) => {
            $$renderer4.push(`50`);
          });
          $$renderer3.option({ value: "100" }, ($$renderer4) => {
            $$renderer4.push(`100`);
          });
          $$renderer3.option({ value: "200" }, ($$renderer4) => {
            $$renderer4.push(`200`);
          });
        },
        "svelte-1wk5kq2"
      );
      $$renderer2.push(`</div> <div class="setting-divider svelte-1wk5kq2"></div> <h3 class="svelte-1wk5kq2">${escape_html(settings_default_config())}</h3> <div class="setting-group svelte-1wk5kq2"><label for="max-depth" class="svelte-1wk5kq2">${escape_html(config_max_depth())}</label> <input id="max-depth" type="number"${attr("value", maxDepth)} min="1" max="50" class="svelte-1wk5kq2"/></div> <div class="setting-group svelte-1wk5kq2"><label for="crawl-time" class="svelte-1wk5kq2">${escape_html(config_time_limit())}</label> <input id="crawl-time" type="number"${attr("value", maxCrawlTime)} min="60" max="86400" class="svelte-1wk5kq2"/></div> <div class="setting-row svelte-1wk5kq2"><label class="checkbox-label svelte-1wk5kq2"><input type="checkbox"${attr("checked", respectRobots, true)} class="svelte-1wk5kq2"/> ${escape_html(config_respect_robots())}</label></div> <div class="setting-row svelte-1wk5kq2"><label class="checkbox-label svelte-1wk5kq2"><input type="checkbox"${attr("checked", checkSitemap, true)} class="svelte-1wk5kq2"/> ${escape_html(config_check_sitemap())}</label></div> <div class="setting-row svelte-1wk5kq2"><label class="checkbox-label svelte-1wk5kq2"><input type="checkbox"${attr("checked", checkSemantics, true)} class="svelte-1wk5kq2"/> ${escape_html(config_check_semantics())}</label></div></div> <div class="modal-footer svelte-1wk5kq2"><button class="btn btn-secondary svelte-1wk5kq2">${escape_html(settings_cancel())}</button> <button class="btn btn-primary svelte-1wk5kq2"${attr("disabled", saving, true)}>${escape_html(settings_save())}</button></div></div></div>`);
    } else {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]-->`);
    bind_props($$props, { open });
  });
}
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let projects = [];
    let selectedProjectId = "";
    let newProjectName = "";
    let renamingProjectId = "";
    let renamingName = "";
    let detailPageId = "";
    let sidebarOpen = false;
    let settingsModalOpen = false;
    let $$settled = true;
    let $$inner_renderer;
    function $$render_inner($$renderer3) {
      $$renderer3.push(`<div class="app-layout svelte-1uha8ag"><button class="hamburger svelte-1uha8ag" aria-label="Toggle menu">${escape_html("☰")}</button> `);
      {
        $$renderer3.push("<!--[-1-->");
      }
      $$renderer3.push(`<!--]--> <aside${attr_class("sidebar svelte-1uha8ag", void 0, { "open": sidebarOpen })}><div class="sidebar-header svelte-1uha8ag"><h1 class="logo svelte-1uha8ag">${escape_html(app_title())}</h1> <button class="btn-settings svelte-1uha8ag" aria-label="Settings">⚙️</button></div> <div class="project-create svelte-1uha8ag"><input type="text"${attr("value", newProjectName)}${attr("placeholder", sidebar_new_project_placeholder())} class="svelte-1uha8ag"/> <button class="btn-icon svelte-1uha8ag"${attr("disabled", !newProjectName.trim(), true)}>+</button></div> <nav class="project-list svelte-1uha8ag"><!--[-->`);
      const each_array = ensure_array_like(projects);
      for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
        let project = each_array[$$index];
        $$renderer3.push(`<div${attr_class("project-item svelte-1uha8ag", void 0, { "selected": project.id === selectedProjectId })} role="button" tabindex="0">`);
        if (renamingProjectId === project.id) {
          $$renderer3.push("<!--[0-->");
          $$renderer3.push(`<input type="text"${attr("value", renamingName)} class="rename-input svelte-1uha8ag"/>`);
        } else {
          $$renderer3.push("<!--[-1-->");
          $$renderer3.push(`<span class="project-name svelte-1uha8ag">${escape_html(project.name)}</span> <span class="project-date svelte-1uha8ag">${escape_html(new Date(project.created_at).toLocaleDateString())}</span> <div class="project-actions svelte-1uha8ag"><button class="btn-mini svelte-1uha8ag"${attr("title", sidebar_rename())}>✎</button> <button class="btn-mini btn-mini-danger svelte-1uha8ag"${attr("title", sidebar_delete())}>×</button></div>`);
        }
        $$renderer3.push(`<!--]--></div>`);
      }
      $$renderer3.push(`<!--]--> `);
      if (projects.length === 0) {
        $$renderer3.push("<!--[0-->");
        $$renderer3.push(`<div class="empty-projects svelte-1uha8ag">${escape_html(sidebar_no_projects())}</div>`);
      } else {
        $$renderer3.push("<!--[-1-->");
      }
      $$renderer3.push(`<!--]--></nav></aside> <main class="main-content svelte-1uha8ag">`);
      {
        $$renderer3.push("<!--[0-->");
        $$renderer3.push(`<div class="no-project svelte-1uha8ag"><h2 class="svelte-1uha8ag">${escape_html(app_select_project())}</h2> <p>${escape_html(app_select_project_hint())}</p></div>`);
      }
      $$renderer3.push(`<!--]--></main></div> `);
      PageDetailPanel($$renderer3, {
        onClose: () => detailPageId = "",
        get pageId() {
          return detailPageId;
        },
        set pageId($$value) {
          detailPageId = $$value;
          $$settled = false;
        }
      });
      $$renderer3.push(`<!----> `);
      SettingsModal($$renderer3, {
        get open() {
          return settingsModalOpen;
        },
        set open($$value) {
          settingsModalOpen = $$value;
          $$settled = false;
        }
      });
      $$renderer3.push(`<!---->`);
    }
    do {
      $$settled = true;
      $$inner_renderer = $$renderer2.copy();
      $$render_inner($$inner_renderer);
    } while (!$$settled);
    $$renderer2.subsume($$inner_renderer);
  });
}
export {
  _page as default
};
