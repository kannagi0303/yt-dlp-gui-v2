pub fn text<'a>(key: &'a str) -> &'a str {
    match key {
        "tab.advanced" => "متقدم",
        "tab.about" => "About",
        "about.tools" => "إصدارات الأدوات",
        "about.current_version" => "الحالي",
        "about.latest_version" => "الأحدث",
        "about.author" => "المؤلف",
        "about.source" => "المصدر",
        "about.status" => "الحالة",
        "about.message" => "الرسالة",
        "about.check_updates" => "التحقق من التحديثات",
        "about.update_all" => "تحديث الكل",
        "about.restart" => "إعادة التشغيل",
        "about.open_release" => "فتح Release",
        "about.install" => "تثبيت",
        "about.update" => "تحديث",
        "about.running" => "جارٍ التحقق من التحديثات...",
        "about.last_check" => "آخر تحقق:",
        "about.relative.minutes" => "{count} د",
        "about.relative.hour" => "ساعة واحدة",
        "about.relative.hours" => "{count} ساعات",
        "about.relative.day" => "يوم واحد",
        "about.relative.days" => "{count} أيام",
        "about.never_checked" => "لم يتم التحقق من التحديثات بعد",
        "about.no_release_notes_loaded" => {
            "لم يتم تحميل ملاحظات الإصدار. اضغط التحقق من التحديثات أولاً."
        }
        "about.ownership.managed_portable" => "مُدار بواسطة v2",
        "about.ownership.external" => "خارجي",
        "about.ownership.missing" => "مفقود",
        "about.ownership.unknown" => "غير معروف",
        "about.status.unknown" => "لم يتم التحقق",
        "about.status.checking" => "جارٍ التحقق",
        "about.status.up_to_date" => "الأحدث ✓",
        "about.status.update_available" => "تحديث متاح ↑",
        "about.status.missing" => "مفقود +",
        "about.status.downloading" => "جارٍ التنزيل",
        "about.status.downloading_percent" => "جارٍ التنزيل {percent}%",
        "about.status.staged" => "جاهز للتطبيق",
        "about.status.pending_restart" => "بانتظار إعادة التشغيل",
        "about.status.applying" => "جارٍ التطبيق",
        "about.status.installed" => "مثبت",
        "about.status.skipped" => "تم التخطي",
        "about.status.failed" => "فشل !",
        "tab.options" => "الخيارات",
        "tab.log" => "السجل",
        "main.url_hint" => "URL",
        "action.download" => "تنزيل",
        "action.add" => "إضافة",
        "action.analyze" => "تحليل",
        "action.stop" => "إيقاف",
        "action.stopping" => "جارٍ الإيقاف",
        "action.cut" => "قص",
        "action.copy" => "نسخ",
        "action.paste" => "لصق",
        "action.clear" => "مسح",
        "item.thumbnail" => "صورة مصغّرة",
        "item.thumbnail_preview" => "معاينة الصورة المصغّرة",
        "single.title" => "العنوان",
        "single.description" => "الوصف",
        "single.info.channel" => "القناة",
        "single.info.date" => "التاريخ",
        "single.info.views" => "المشاهدات",
        "item.download_thumbnail" => "تنزيل الصورة المصغرة",
        "media.video" => "فيديو",
        "media.audio" => "الصوت",
        "media.subtitle" => "الترجمات",
        "media.section" => "النطاق",
        "item.file_name" => "اسم الملف",
        "main.target_folder" => "مجلد الإخراج",
        "picker.title.video" => "اختيار صيغة الفيديو",
        "picker.title.audio" => "اختيار صيغة الصوت",
        "picker.title.subtitle" => "اختيار الترجمات",
        "picker.title.section" => "اختيار النطاق",
        "action.back" => "رجوع",
        "picker.mode.filter" => "عوامل التصفية",
        "picker.mode.table" => "جدول",
        "action.confirm" => "تأكيد",
        "picker.empty_table" => "لا توجد صيغ لعرضها",
        "picker.header.resolution" => "الدقة",
        "picker.header.range" => "النطاق",
        "picker.header.fps" => "FPS",
        "picker.header.format" => "الصيغة",
        "picker.header.codec" => "الترميز",
        "picker.header.size" => "الحجم",
        "picker.header.sample_rate" => "معدل العينة",
        "picker.filter.resolution" => "الدقة",
        "picker.filter.range" => "النطاق",
        "picker.filter.fps" => "FPS",
        "picker.filter.codec" => "الترميز",
        "picker.filter.sample_rate" => "معدل العينة",
        "main.missing_yt_dlp_callout" => "yt-dlp غير موجود. ثبّته أو اختر yt-dlp.exe من الخيارات.",
        "advance.source" => "المصدر",
        "advance.config" => "الإعدادات",
        "advance.none" => "لا شيء",
        "advance.network_access" => "الشبكة والوصول",
        "advance.proxy" => "الوكيل",
        "advance.enable_proxy" => "تفعيل الوكيل",
        "advance.certificate" => "الشهادة",
        "advance.skip_certificate_verification" => "تخطي التحقق من الشهادة",
        "advance.use_cookies" => "استخدام cookies",
        "advance.enable_cookies" => "تفعيل cookies",
        "advance.cookie_source" => "مصدر cookies",
        "advance.cookie_source.auto" => "تلقائي حسب الموقع",
        "advance.cookie_source.file" => "استخدام ملف (cookies.txt)",
        "advance.cookie_auto" => "تلقائي",
        "advance.cookie_auto_note" => "تستخدم التنزيلات ملف Cookie المحفوظ المطابق للرابط.",
        "advance.cookie_rescue" => "استرداد Cookie",
        "advance.cookie_file" => "ملف cookies",
        "advance.get_cookie" => "الحصول على Cookie",
        "advance.cookie" => "Cookie",
        "advance.cookie.off" => "Do not use",
        "advance.cookie.browser" => "Browser",
        "advance.cookie.file" => "File",
        "advance.cookie_file_source" => "File source",
        "advance.cookie_file_custom" => "Custom cookies.txt",
        "advance.cookie_file_auto_select" => "Auto select",
        "advance.cookie_manager_row" => "Management",
        "advance.manage_cookie" => "Manage Cookie",
        "advance.cookie_manager_title" => "Cookie Manager",
        "advance.add_cookie" => "Add Cookie",
        "advance.cookie_manager_empty" => "No Cookie files have been added.",
        "advance.cookie_manager_name" => "Name",
        "advance.cookie_manager_updated" => "Updated",
        "advance.cookie_manager_actions" => "Actions",
        "advance.cookie_manager_refresh" => "Reacquire",
        "advance.cookie_manager_delete" => "Delete",
        "advance.file" => "File",
        "youtube_login_rescue.short_note" => "افتح نافذة متصفح مخصصة للحصول على ملفات Cookie.",
        "youtube_login_rescue.title" => "استرداد Cookie",
        "youtube_login_rescue.confirm_heading" => "فتح نافذة تسجيل دخول مخصصة",
        "youtube_login_rescue.confirm_body" => {
            "ستفتح نافذة {browser} مستقلة الرابط بدون استخدام بيانات المتصفح الشخصية."
        }
        "youtube_login_rescue.target_url_label" => "رابط الموقع",
        "youtube_login_rescue.target_url_hint" => "https://www.youtube.com/",
        "youtube_login_rescue.clipboard_prefilled" => "تم ملء الرابط من الحافظة.",
        "youtube_login_rescue.drop_url_note" => "الصق رابطاً أو اسحب ملف .url / ملفاً نصياً.",
        "youtube_login_rescue.paste_clipboard" => "لصق من الحافظة",
        "youtube_login_rescue.cookie_note" => {
            "سجّل الدخول هناك. عند العثور على ملفات Cookie، ستُغلق النافذة وتُطبق تلقائياً."
        }
        "youtube_login_rescue.no_browser_title" => "لم يتم العثور على متصفح مدعوم",
        "youtube_login_rescue.no_browser_body" => {
            "الحصول على ملفات Cookie يحتاج حالياً إلى Chrome أو Brave أو Microsoft Edge. لا يزال بإمكانك اختيار cookies.txt يدوياً."
        }
        "youtube_login_rescue.start" => "بدء",
        "youtube_login_rescue.opening" => "جارٍ فتح {browser}...",
        "youtube_login_rescue.waiting_for_cdp" => {
            "بانتظار اتصال نافذة تسجيل الدخول في {browser}..."
        }
        "youtube_login_rescue.waiting_for_cookie" => {
            "نافذة تسجيل الدخول متصلة. بانتظار ملفات Cookie من الموقع..."
        }
        "youtube_login_rescue.cookie_exported" => "تم حفظ Cookie.",
        "youtube_login_rescue.cookie_exported_note" => {
            "تم حفظ Cookie لـ {site}. ستستخدم التنزيلات من هذا الموقع هذا الملف تلقائياً."
        }
        "youtube_login_rescue.do_not_close_note" => {
            "يرجى إبقاء متصفح تسجيل الدخول مفتوحاً أثناء تشغيل هذا الفحص."
        }
        "youtube_login_rescue.cdp_ready" => "نافذة تسجيل الدخول متصلة.",
        "youtube_login_rescue.ready_next_step_note" => {
            "يرجى إكمال تسجيل الدخول إلى YouTube في المتصفح. ستتم إضافة تصدير Cookie في الخطوة التالية."
        }
        "youtube_login_rescue.close_login_window" => "إغلاق نافذة تسجيل الدخول",
        "youtube_login_rescue.failed" => "فشل استرداد Cookie",
        "youtube_login_rescue.retry" => "إعادة المحاولة",
        "advance.no_cookies_txt_selected" => "لم يتم اختيار cookies.txt",
        "advance.browse" => "تصفح",
        "advance.select_netscape_cookies_txt" => "اختيار Netscape cookies.txt",
        "advance.clear" => "مسح",
        "advance.browser" => "المتصفح",
        "advance.default" => "افتراضي",
        "advance.external_downloader" => "منزّل خارجي",
        "advance.use_aria2_for_faster_downloads" => "استخدام Aria2 لتنزيل أسرع",
        "advance.download_control" => "التحكم بالتنزيل",
        "advance.concurrent_fragments" => "الأجزاء المتزامنة",
        "advance.1_default" => "1 (افتراضي)",
        "advance.rate_limit" => "حد السرعة",
        "advance.e_g_2m_800k_leave_empty_for_unlimited" => "مثلاً 2M، 800K؛ اتركه فارغًا بلا حد",
        "advance.chapters" => "الفصول",
        "advance.chapter_download_compatibility_mode" => "وضع توافق تنزيل الفصول",
        "advance.file_time" => "وقت الملف",
        "advance.file_time.none" => "عدم التغيير",
        "advance.file_time.upload_date" => "استخدام تاريخ رفع الفيديو",
        "advance.file_time.download_time" => "استخدام وقت التنزيل",
        "advance.post_processing" => "المعالجة اللاحقة",
        "advance.thumbnail" => "الصورة المصغرة",
        "advance.download" => "تنزيل",
        "advance.embed" => "تضمين",
        "advance.subtitles" => "الترجمات",
        "advance.download_conversion" => "التحويل بعد التنزيل",
        "advance.enable" => "تفعيل",
        "advance.settings" => "الإعدادات",
        "item.save_as" => "حفظ باسم",
        "item.error" => "خطأ",
        "item.all" => "الكل",
        "item.queued" => "في الانتظار",
        "item.done" => "تم",
        "item.failed" => "فشل",
        "item.clear_all" => "مسح الكل",
        "item.add_a_video_url" => "إضافة رابط فيديو",
        "item.add_an_audio_url" => "إضافة رابط صوت",
        "item.after_adding_choose_the_video_format_here" => "اختيار تنسيق الفيديو",
        "item.after_adding_choose_the_audio_format_here" => "اختيار تنسيق الصوت",
        "item.loading_thumbnail" => "جارٍ تحميل الصورة المصغّرة",
        "item.file_actions" => "إجراءات الملف",
        "item.open_file" => "فتح الملف",
        "item.open_folder" => "فتح المجلد",
        "item.copy_path" => "نسخ المسار",
        "item.file_not_found_opened_the_output_location" => {
            "لم يتم العثور على الملف؛ تم فتح موقع الإخراج."
        }
        "item.opened_output_location" => "تم فتح موقع الإخراج.",
        "item.copied_output_path" => "تم نسخ مسار الإخراج.",
        "prepare.language" => "اللغة",
        "prepare.back" => "رجوع",
        "prepare.auto_detect" => "اكتشاف تلقائي",
        "prepare.install_the_required_tools_now_or_skip_and_h" => {
            "ثبّت الأدوات المطلوبة الآن، أو تخطَّ ذلك واضبطها لاحقًا من الخيارات."
        }
        "prepare.optional" => "اختياري",
        "prepare.missing" => "مفقود",
        "prepare.install_later" => "التثبيت لاحقًا",
        "prepare.downloading_100" => "التنزيل 100%",
        "prepare.extracting_100" => "الاستخراج 100%",
        "prepare.install_failed" => "فشل التثبيت",
        "prepare.install_all" => "تثبيت الكل",
        "prepare.reinstall" => "إعادة التثبيت",
        "prepare.installing" => "جارٍ التثبيت",
        "prepare.skip" => "تخطي",
        "prepare.install" => "تثبيت",
        "prepare.another_tool_is_already_being_installed" => "هناك أداة أخرى قيد التثبيت بالفعل.",
        "prepare.needs_attention" => "يحتاج إلى انتباه",
        "prepare.req.app_folder.title" => "مجلد التطبيق",
        "prepare.req.app_folder.description" => {
            "يجب أن يكون مجلد النسخة المحمولة قابلاً للكتابة لحفظ الإعدادات وبيانات الدعم."
        }
        "prepare.req.tools_folder.title" => "مجلد الأدوات",
        "prepare.req.tools_folder.description" => {
            "يتم تخزين yt-dlp وFFmpeg وDeno هنا عند نشر الأدوات المطلوبة."
        }
        "prepare.req.deployment_temp.title" => "مجلد مؤقت للنشر",
        "prepare.req.deployment_temp.description" => {
            "يستخدم استخراج FFmpeg وDeno هذا المجلد المؤقت."
        }
        "prepare.req.download_cache.title" => "ذاكرة التخزين المؤقت للتنزيل",
        "prepare.req.download_cache.description" => {
            "يحفظ وضع التخزين المؤقت في yt-dlp-gui ذاكرة yt-dlp المؤقتة هنا."
        }
        "prepare.req.output_folder.title" => "مجلد الإخراج",
        "prepare.req.output_folder.description" => "يتم حفظ الفيديوهات والصوت والترجمات هنا.",
        "prepare.req.output_folder.recommendation" => {
            "اختر مجلد إخراج صالحاً من الشاشة الرئيسية أو من الخيارات."
        }
        "prepare.req.config_file.title" => "ملف الإعدادات",
        "prepare.req.config_file.description" => {
            "يجب أن يتمكن التطبيق من حفظ حالة تخطي صفحة Prepare ومسارات الأدوات."
        }
        "prepare.req.generic_writable_recommendation" => {
            "اختر مجلداً قابلاً للكتابة وتحقق من الأذونات."
        }
        "prepare.req.config_not_folder" => "يشير مسار الإعدادات إلى مجلد. اختر مسار ملف بدلاً من ذلك.",
        "prepare.req.config_readonly" => "ملف الإعدادات للقراءة فقط.",
        "prepare.req.config_readonly_recommendation" => {
            "اسمح بالكتابة إلى ملف الإعدادات أو اختر مجلداً آخر للتطبيق."
        }
        "prepare.req.use_folder_path" => "اختر مسار مجلد بدلاً من مسار ملف.",
        "prepare.req.move_portable_folder" => "انقل التطبيق إلى مجلد محمول قابل للكتابة.",
        "prepare.req.avoid_protected_folder" => {
            "لا تضع التطبيق المحمول داخل Program Files أو مجلد Windows. انقله إلى D:\\Portable أو إلى مجلد المستخدم."
        }
        "prepare.req.move_non_synced_folder" => {
            "انقله إلى مجلد غير متزامن، مثل D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.drive_parent_exists" => "تأكد من وجود محرك الأقراص والمجلد الأب.",
        "prepare.req.permission_denied" => {
            "انقل التطبيق إلى مجلد محمول قابل للكتابة. إذا استمر الفشل في سطح المكتب/المستندات/التنزيلات، فقد يكون وصول المجلدات المتحكم به في Defender هو السبب."
        }
        "prepare.req.file_in_use" => "أغلق البرنامج الذي قد يستخدم هذا المجلد، أو اختر مجلداً آخر.",
        "prepare.req.free_disk_space" => "حرر مساحة على القرص أو اختر قرصاً آخر.",
        "prepare.req.path_too_long" => {
            "انقل التطبيق إلى مسار أقصر، مثل D:\\Portable\\yt-dlp-gui-v2."
        }
        "prepare.req.choose_writable_portable_folder" => {
            "اختر مجلداً محمولاً قابلاً للكتابة بوضوح ثم تحقق مرة أخرى."
        }
        "prepare.req.clear_write_test" => "احذف ملف اختبار الكتابة المتبقي ثم تحقق مرة أخرى.",
        "options.this_url_contains_both_a_video_and_a_playlis" => {
            "هذا الرابط يحتوي على فيديو وقائمة تشغيل"
        }
        "options.detected" => "تم اكتشاف ",
        "options.playlist_prompt" => "سؤال قائمة التشغيل",
        "options.which_one_should_be_loaded" => "أيهما يجب تحميله؟",
        "options.both_video_and_playlist_were_detected" => "تم اكتشاف فيديو وقائمة تشغيل",
        "options.this_playlist_may_contain_many_items" => {
            "قد تحتوي قائمة التشغيل هذه على عناصر كثيرة."
        }
        "options.playlist_risk.kind.channel_generated" => "قائمة قناة مولّدة من YouTube",
        "options.playlist_risk.kind.youtube_mix_radio" => "YouTube Mix / Radio",
        "options.playlist_risk.kind.youtube_music_album" => "ألبوم/مجموعة YouTube Music",
        "options.playlist_risk.kind.liked_videos" => "الفيديوهات التي أعجبتك",
        "options.playlist_risk.kind.favorites_legacy" => "قائمة مفضلات قديمة",
        "options.playlist_risk.note.channel_generated" => {
            "تعامل مع قائمة القناة المولّدة من YouTube بحذر."
        }
        "options.playlist_risk.note.youtube_mix_radio" => {
            "قد تحتوي قائمة Mix / Radio هذه على عناصر كثيرة وقد تتغير مع الوقت."
        }
        "options.playlist_risk.note.youtube_music_album" => {
            "عادةً يكون هذا ألبومًا أو مجموعة في YouTube Music."
        }
        "options.playlist_risk.note.liked_videos" => {
            "الفيديوهات التي أعجبتك تتطلب غالبًا تسجيل الدخول أو cookies."
        }
        "options.playlist_risk.note.favorites_legacy" => {
            "هذا نمط قديم لقائمة المفضلات وقد لا يكون مستقرًا الآن."
        }
        "options.video" => "فيديو",
        "options.playlist" => "قائمة التشغيل",
        "options.cancel" => "إلغاء",
        "options.load" => "تحميل",
        "options.behavior" => "السلوك",
        "options.add_action" => "إجراء الإضافة",
        "options.download_directly" => "تنزيل مباشرة",
        "options.clipboard_change" => "تغيّر الحافظة",
        "options.run_immediately" => "تشغيل فورًا",
        "options.tabs" => "علامات التبويب",
        "options.log_tab" => "تبويب السجل",
        "options.show_log_tab" => "إظهار السجل",
        "options.playlist_2" => "قائمة التشغيل",
        "options.with_playlist" => "مع قائمة التشغيل",
        "options.ask" => "اسأل",
        "options.single_video" => "فيديو واحد",
        "options.full_playlist" => "قائمة التشغيل كاملة",
        "options.high_risk_prompt" => "تنبيه عالي الخطورة",
        "options.on" => "تشغيل",
        "options.playlist_count" => "عدد عناصر قائمة التشغيل",
        "options.limit" => "الحد",
        "options.max" => "الحد الأقصى:",
        "options.items" => " عناصر",
        "options.language" => "اللغة",
        "options.current_language" => "اللغة الحالية",
        "options.back" => "رجوع",
        "options.choose" => "اختيار",
        "options.auto_detect" => "اكتشاف تلقائي",
        "options.tool_paths" => "مسارات الأدوات",
        "options.file_actions" => "إجراءات الملف",
        "options.action_button" => "زر الإجراء",
        "options.file_action.show_menu" => "إظهار القائمة",
        "options.cache" => "ذاكرة التخزين المؤقت",
        "options.cache_location" => "موقع الذاكرة المؤقتة",
        "options.cache_location.default" => "افتراضي",
        "options.cache_usage" => "الاستخدام",
        "options.cache_usage_detail" => "الإجمالي: {total} · الصوت: {audio} · المنتهي: {expired}",
        "options.cache_cleanup" => "تنظيف",
        "options.cache_refresh" => "تحديث",
        "options.cache_clear_expired" => "مسح المنتهية",
        "options.cache_clear_audio" => "مسح الصوت",
        "options.cache_clear_all" => "مسح الكل",
        "options.appearance_window" => "المظهر والنافذة",
        "options.notifications" => "الإشعارات",
        "options.enable" => "تمكين",
        "options.theme" => "السمة",
        "options.theme_mode.system" => "اتباع النظام",
        "options.theme_mode.light" => "فاتح",
        "options.theme_mode.dark" => "داكن",
        "options.theme_color" => "لون السمة",
        "options.theme_color.off" => "إيقاف",
        "options.theme_color.blue" => "أزرق",
        "options.theme_color.soft_blue" => "أزرق ناعم",
        "options.theme_color.purple" => "أرجواني",
        "options.theme_color.pink" => "وردي",
        "options.theme_color.green" => "أخضر",
        "options.theme_color.orange" => "برتقالي",
        "options.theme_color.slate" => "رمادي داكن",
        "options.ui_scale" => "مقياس الواجهة",
        "options.apply" => "تطبيق",
        "options.current" => "الحالي",
        "options.always_on_top" => "دائمًا في الأعلى",
        "options.window_position" => "موضع النافذة",
        "options.remember" => "تذكر",
        "options.window_size" => "حجم النافذة",
        "options.reinstall" => "إعادة التثبيت",
        "options.installing" => "جارٍ التثبيت",
        "options.install" => "تثبيت",
        "options.executable" => "ملف تنفيذي",
        "main.controlled_by_config" => "تتحكم به الإعدادات: ",
        "main.controlled_by_config_2" => "تتحكم به الإعدادات",
        "picker.no_chapters_available" => "لا توجد فصول متاحة.",
        "picker.choose_the_range_to_download_for_this_item_d" => {
            "اختر النطاق المراد تنزيله لهذا العنصر. الافتراضي هو الفيديو الكامل."
        }
        "picker.chapter_compatibility_mode_is_on_chapter_dow" => {
            "وضع توافق الفصول مفعّل: ستستخدم تنزيلات الفصول صيغة ملف واحدة أكثر استقرارًا."
        }
        "picker.subtitles_will_not_be_downloaded" => "لن يتم تنزيل الترجمات.",
        "picker.no_subtitles_are_available_for_this_video" => "لا توجد ترجمات متاحة لهذا الفيديو.",
        "picker.no_subtitles_are_available_in_this_tab" => "لا توجد ترجمات متاحة في هذا التبويب.",
        "picker.source_language" => "لغة المصدر",
        "picker.translation_target" => "هدف الترجمة",
        "picker.tip_youtube_auto_translated_subtitles_are_mo" => {
            "نصيحة: الترجمات التلقائية من YouTube أكثر عرضة للتقييد من الترجمات الأصلية. اختر «بدون ترجمة» إذا كنت تحتاج النص الأصلي فقط."
        }
        "picker.no_subtitles_are_available_for_this_source" => "لا توجد ترجمات متاحة لهذا المصدر.",
        "picker.target" => "الهدف",
        "picker.available_subtitles" => "الترجمات المتاحة",
        "picker.language" => "اللغة",
        "picker.subtitle_tab.none" => "بدون ترجمات",
        "picker.subtitle_tab.original" => "الترجمات الأصلية",
        "picker.subtitle_tab.automatic" => "الترجمات التلقائية",
        "picker.waiting_analysis" => "بانتظار التحليل",
        "picker.audio_from_video" => "يُحدَّد حسب صيغة الفيديو",
        "picker.not_selected" => "غير محدد",
        "picker.full_video" => "الفيديو الكامل",
        "picker.no_translation" => "بدون ترجمة",
        "picker.until_end" => "النهاية",
        "prepare.status.ready" => "جاهز",
        "prepare.status.missing" => "مفقود",
        "prepare.status.warning" => "يحتاج إلى انتباه",
        "prepare.status.failed" => "فشل",
        "tool_install.stage.preparing" => "التحضير",
        "tool_install.stage.downloading" => "التنزيل",
        "tool_install.stage.extracting" => "الاستخراج",
        "tool_install.stage.installing" => "جارٍ التثبيت",
        "tool_install.stage.completed" => "اكتمل",
        "tool_install.stage.failed" => "فشل",
        "item.status.queued" => "في قائمة الانتظار",
        "item.status.running" => "قيد التشغيل",
        "item.status.finished" => "تم",
        "item.status.failed" => "فشل",
        "item.status.cancelled" => "أُلغي",
        "processing.transcode" => "تحويل الترميز",
        "transcode.graph.axis.compatibility" => "التوافق",
        "transcode.graph.axis.capacity" => "السعة",
        "transcode.graph.axis.resolution" => "الدقة",
        "transcode.graph.axis.format" => "الصيغة",
        "transcode.graph.compatibility_scope" => "نطاق التوافق",
        "transcode.graph.capacity_target" => "هدف الحجم",
        "transcode.graph.resolution_limit" => "حد الدقة",
        "transcode.graph.format_goal" => "هدف الصيغة",
        "processing.video" => "الفيديو",
        "processing.audio" => "الصوت",
        "processing.container" => "الحاوية",
        "processing.subtitle" => "الترجمات",
        "processing.choice.source" => "الأصلي",
        "processing.subtitle.preserve" => "الأصلي",
        "processing.subtitle.embed" => "تضمين",
        "processing.subtitle.burn" => "حرق داخل الفيديو",
        "advance.filter_netscape_cookies_txt" => "Netscape cookies.txt",
        "advance.filter_all_files" => "كل الملفات",
        "options.filter_executable" => "ملف تنفيذي",
        "app_mode.origin" => "وضع Origin",
        "app_mode.standard" => "الوضع القياسي",
        "app_mode.audio" => "وضع الصوت",
        "music.status.completed" => "تم",
        "music.status.resolving" => "جارٍ التحليل",
        "music.status.buffering" => "جارٍ التخزين المؤقت",
        "music.status.ready" => "جاهز",
        "music.status.caching" => "جارٍ التخزين في الذاكرة المؤقتة",
        "music.status.playing" => "يعمل",
        "music.status.paused" => "متوقف مؤقتًا",
        "music.status.failed" => "فشل",
        "notification.download_complete" => "اكتمل التنزيل",
        "notification.download_failed" => "فشل التنزيل",
        "notification.completed_file" => "اكتمل: {file}",
        "notification.download_completed" => "اكتمل التنزيل.",
        "options.music_download_format" => "Music download format",
        "options.music_download_audio_label" => "إخراج الصوت",
        "options.music_download_preference_best" => "الأفضل",
        _ => key,
    }
}
