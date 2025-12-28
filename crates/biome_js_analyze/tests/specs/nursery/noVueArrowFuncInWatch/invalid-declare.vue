<script>
import { declareComponent } from 'vue';

/* should generate diagnostics */
export default declareComponent({
  watch: {
    foo: (val, oldVal) => {
      console.log('new: %s, old: %s', val, oldVal)
    },
    bar: {
			handler: (val, oldVal) => {
				console.log('new: %s, old: %s', val, oldVal)
			}
    }
    short: (val, oldVal) => console.log('new: %s, old: %s', val, oldVal)
  }
})
</script>
