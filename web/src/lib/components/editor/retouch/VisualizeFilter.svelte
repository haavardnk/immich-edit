<script lang="ts" module>
  export const VISUALIZE_FILTER = 'retouch-visualize';
</script>

<svg class="pointer-events-none absolute size-0" aria-hidden="true">
  <filter id={VISUALIZE_FILTER} color-interpolation-filters="sRGB">
    <feColorMatrix
      in="SourceGraphic"
      type="matrix"
      values="0.2126 0.7152 0.0722 0 0 0.2126 0.7152 0.0722 0 0 0.2126 0.7152 0.0722 0 0 0 0 0 1 0"
      result="luma"
    />
    <feColorMatrix
      in="luma"
      type="matrix"
      values="-1 0 0 0 1 0 -1 0 0 1 0 0 -1 0 1 0 0 0 1 0"
      result="lumaInverse"
    />
    <feGaussianBlur in="luma" stdDeviation="2.5" result="blurred" />
    <feColorMatrix
      in="blurred"
      type="matrix"
      values="1 0 0 0 0 0 1 0 0 0 0 0 1 0 0 0 0 0 0 1"
      result="blur"
    />
    <feColorMatrix
      in="blurred"
      type="matrix"
      values="-1 0 0 0 1 0 -1 0 0 1 0 0 -1 0 1 0 0 0 0 1"
      result="blurInverse"
    />
    <feComposite
      in="luma"
      in2="blurInverse"
      operator="arithmetic"
      k2="8"
      k3="8"
      k4="-8"
      result="above"
    />
    <feComposite
      in="blur"
      in2="lumaInverse"
      operator="arithmetic"
      k2="8"
      k3="8"
      k4="-8"
      result="below"
    />
    <feComposite in="above" in2="below" operator="arithmetic" k2="1" k3="1" />
  </filter>
</svg>
