<script setup lang="ts">
const { back = false } = defineProps<{
  /**
   * Should we show a back button?
   * Note: this will be forced to false on xl screens to avoid duplicating the sidebar's back button.
   */
  back?: boolean | "small-only";
  /** Show the back button on small screens */
  backOnSmallScreen?: boolean;
  /** Do not applying overflow hidden to let use floatable components in title */
  noOverflowHidden?: boolean;
}>();

const container = ref();
const route = useRoute();
const userSettings = useUserSettings();
const { height: windowHeight } = useWindowSize();
const { height: containerHeight } = useElementBounding(container);
const wideLayout = computed(() => route.meta.wideLayout ?? false);
const sticky = computed(() => route.path?.startsWith("/settings/"));
const containerClass = computed(() => {
  // we keep original behavior when not in settings page and when the window height is smaller than the container height
  if (
    !isHydrated.value ||
    !sticky.value ||
    windowHeight.value < containerHeight.value
  )
    return null;

  return "lg:sticky lg:top-0";
});

const showBackButton = computed(() => {
  switch (back) {
    case "small-only":
      return isSmallOrMediumScreen.value;
    case true:
      return !isExtraLargeScreen.value;
    default:
      return false;
  }
});
</script>

<template lang="hsml">
div(ref="container" :class="containerClass")
  .sticky.top-0.z-20(
    pt="[env(safe-area-inset-top,0)]"
    bg="[rgba(var(--rgb-bg-base),0.7)]"
    :class="{
      'backdrop-blur': !getPreferences(userSettings, 'optimizeForLowPerformanceDevice'),
    }"
  )
    .min-h-53px.px-2.py-1(flex="~ justify-between" :class="{ 'xl:hidden': $route.name !== 'tag' }" border="b base")
      .w-full(flex="~ items-center")
        button.btn-text.flex.items-center.p-3.xl:hidden(
          v-if="backOnSmallScreen || showBackButton"
          :aria-label="$t('nav.back')"
          @click="$router.go(-1)"
        )
          .text-lg.rtl-flip(i-ri:arrow-left-line)
        .flex.w-full
          slot(name="title")
        .sm:hidden.h-7.w-1px
      .px-3(flex="~ items-center shrink-0 gap-x-2")
        slot(name="actions")
        PwaBadge.xl:hidden
        NavUser(v-if="isHydrated")
        NavUserSkeleton(v-else)
    slot(name="header")
      div(hidden)
  PwaInstallPrompt.xl:hidden
  .m-auto(:class="isHydrated && wideLayout ? 'xl:w-full sm:max-w-600px' : 'sm:max-w-600px md:shrink-0'")
    .h-6(hidden :class="{ 'xl:block': $route.name !== 'tag' && !$slots.header }")
    slot
</template>
