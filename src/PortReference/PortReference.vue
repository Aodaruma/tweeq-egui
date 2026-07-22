<script setup lang="ts">
import {ref} from 'vue'

import {
	InputAngle,
	InputButton,
	InputButtonToggle,
	InputCheckbox,
	InputColor,
	InputComplex,
	InputCubicBezier,
	InputDropdown,
	InputDrum,
	InputGroup,
	InputNumber,
	InputPosition,
	InputRadio,
	InputShuffle,
	InputSize,
	InputString,
	InputSwitch,
	InputTime,
	InputTranslate,
	InputVec,
	Parameter,
	ParameterGrid,
	TweeqProvider,
	Viewport,
} from '../components'

const number = ref(0.625)
const numberMin = ref(0)
const numberMax = ref(2)
const numberStep = ref(0.125)
const numberPrecision = ref(4)
const numberBarOrigin = ref(0)
const numberBar = ref(true)
const numberClampMin = ref(false)
const numberClampMax = ref(false)
const numberDisabled = ref(false)
const numberInvalid = ref(false)

const angle = ref(35)
const angleSnap = ref(15)
const angleOffset = ref(0)
const angleDisabled = ref(false)
const angleInvalid = ref(false)

const checkbox = ref(true)
const switchValue = ref(false)
const toggle = ref(false)
const stringValue = ref('Baby salmon')
const dropdown = ref('apple')
const drum = ref('banana')
const radio = ref('cherry')
const options = ['apple', 'banana', 'cherry']
const position = ref<[number, number]>([24, -12])
const translate = ref<[number, number]>([-18, 32])
const vector = ref<[number, number, number]>([1, 2, 3])
const size = ref<[number, number]>([1920, 1080])
const frames = ref(1476)
const color = ref('#296bffff')
const bezier = ref<readonly [number, number, number, number]>([
	0.25, 0.1, 0.25, 1,
])
const shuffled = ref(0.42)
const complex = ref({
	opacity: 0.72,
	rotation: 35,
	visible: true,
	name: 'Layer 1',
})

// InputComplex currently leaks child `modelValue` requirements into its Scheme
// type even though it supplies those values internally. Keep this runtime-valid
// reference case while the upstream generic type is corrected.
const complexScheme: any = {
	opacity: {
		type: 'number' as const,
		label: 'Opacity',
		min: 0,
		max: 1,
		step: 0.01,
	},
	rotation: {
		type: 'number' as const,
		ui: 'angle' as const,
		label: 'Rotation',
		snap: 15,
	},
	visible: {
		type: 'boolean' as const,
		ui: 'checkbox' as const,
		label: 'Visible',
	},
	name: {type: 'string' as const, label: 'Name'},
}

function shuffleNumber(previous: number): number {
	return Number(((previous * 0.61803398875 + 0.271828) % 1).toFixed(4))
}
</script>

<template>
	<TweeqProvider>
		<Viewport class="reference-viewport">
			<main class="reference-page">
				<header class="page-header">
					<h1>Tweeq component reference</h1>
					<p>
						Vue source-of-truth · drag, focus, keyboard and state comparison
					</p>
				</header>

				<section class="reference-section number-section">
					<h2>InputNumber</h2>
					<div class="number-layout">
						<div class="subject-card">
							<label>Live subject</label>
							<InputNumber
								v-model="number"
								:min="numberMin"
								:max="numberMax"
								:step="numberStep || undefined"
								:snap="10"
								:precision="numberPrecision"
								:bar="numberBar ? numberBarOrigin : false"
								:clamp-min="numberClampMin"
								:clamp-max="numberClampMax"
								:disabled="numberDisabled"
								:invalid="numberInvalid"
								:default="0.5"
								suffix=" px"
							/>
							<output>{{ number }}</output>
						</div>

						<ParameterGrid class="props-grid">
							<Parameter label="min">
								<InputNumber v-model="numberMin" :bar="false" :step="0.1" />
							</Parameter>
							<Parameter label="max">
								<InputNumber v-model="numberMax" :bar="false" :step="0.1" />
							</Parameter>
							<Parameter label="step (0 = continuous)">
								<InputNumber
									v-model="numberStep"
									:bar="false"
									:min="0"
									:clamp-min="true"
									:step="0.001"
								/>
							</Parameter>
							<Parameter label="precision">
								<InputNumber
									v-model="numberPrecision"
									:min="0"
									:max="10"
									:step="1"
									:clamp-min="true"
									:clamp-max="true"
								/>
							</Parameter>
							<Parameter label="bar origin">
								<InputNumber
									v-model="numberBarOrigin"
									:bar="false"
									:step="0.1"
								/>
							</Parameter>
							<Parameter label="flags">
								<InputGroup class="flag-group">
									<InputCheckbox v-model="numberBar" label="bar" />
									<InputCheckbox v-model="numberClampMin" label="clamp min" />
									<InputCheckbox v-model="numberClampMax" label="clamp max" />
									<InputCheckbox v-model="numberDisabled" label="disabled" />
									<InputCheckbox v-model="numberInvalid" label="invalid" />
								</InputGroup>
							</Parameter>
						</ParameterGrid>
					</div>

					<div class="state-strip">
						<div>
							<label>bar + stepped + clamped</label
							><InputNumber
								:model-value="0.5"
								:min="0"
								:max="1"
								:step="0.1"
								:bar="0"
							/>
						</div>
						<div>
							<label>below range, unclamped</label
							><InputNumber
								:model-value="-0.25"
								:min="0"
								:max="1"
								:clamp-min="false"
								:bar="0"
							/>
						</div>
						<div>
							<label>unranged</label
							><InputNumber :model-value="120" :bar="false" suffix=" px" />
						</div>
						<div>
							<label>disabled</label
							><InputNumber :model-value="0.5" :min="0" :max="1" disabled />
						</div>
						<div>
							<label>invalid</label
							><InputNumber :model-value="0.5" :min="0" :max="1" invalid />
						</div>
					</div>
				</section>

				<section class="reference-section">
					<h2>InputAngle</h2>
					<div class="angle-layout">
						<div class="angle-wide">
							<label>wide composition</label>
							<InputAngle
								v-model="angle"
								:snap="angleSnap"
								:angle-offset="angleOffset"
								:disabled="angleDisabled"
								:invalid="angleInvalid"
							/>
						</div>
						<div class="angle-compact">
							<label>compact rotary</label>
							<InputAngle
								v-model="angle"
								:snap="angleSnap"
								:angle-offset="angleOffset"
							/>
						</div>
						<ParameterGrid>
							<Parameter label="snap">
								<InputNumber
									v-model="angleSnap"
									:min="1"
									:max="360"
									:step="1"
									suffix="°"
								/>
							</Parameter>
							<Parameter label="angle offset">
								<InputAngle v-model="angleOffset" :snap="15" />
							</Parameter>
							<Parameter label="flags">
								<InputGroup>
									<InputCheckbox v-model="angleDisabled" label="disabled" />
									<InputCheckbox v-model="angleInvalid" label="invalid" />
								</InputGroup>
							</Parameter>
						</ParameterGrid>
					</div>
				</section>

				<section class="reference-section">
					<h2>Boolean, action and choice inputs</h2>
					<div class="component-grid">
						<div>
							<label>InputButton</label
							><InputButton label="Action" icon="mdi:play" />
						</div>
						<div>
							<label>InputButtonToggle</label
							><InputButtonToggle v-model="toggle" label="Toggle" />
						</div>
						<div>
							<label>InputCheckbox</label
							><InputCheckbox v-model="checkbox" label="Checkbox" />
						</div>
						<div>
							<label>InputSwitch</label
							><InputSwitch v-model="switchValue" label="Switch" />
						</div>
						<div>
							<label>InputString</label><InputString v-model="stringValue" />
						</div>
						<div>
							<label>InputDropdown</label
							><InputDropdown v-model="dropdown" :options="options" />
						</div>
						<div>
							<label>InputDrum</label
							><InputDrum v-model="drum" :options="options" />
						</div>
						<div class="wide-control">
							<label>InputRadio</label
							><InputRadio v-model="radio" :options="options" />
						</div>
					</div>
				</section>

				<section class="reference-section">
					<h2>Vector, media and advanced inputs</h2>
					<ParameterGrid>
						<Parameter label="InputPosition"
							><InputPosition v-model="position" :min="-100" :max="100"
						/></Parameter>
						<Parameter label="InputTranslate"
							><InputTranslate v-model="translate" :min="-100" :max="100"
						/></Parameter>
						<Parameter label="InputVec"
							><InputVec v-model="vector" :step="0.1"
						/></Parameter>
						<Parameter label="InputSize"
							><InputSize v-model="size"
						/></Parameter>
						<Parameter label="InputTime"
							><InputTime
								v-model="frames"
								:frame-rate="24"
								:min="0"
								:max="100000"
						/></Parameter>
						<Parameter label="InputColor"
							><InputColor v-model="color" alpha
						/></Parameter>
						<Parameter label="InputCubicBezier"
							><InputCubicBezier v-model="bezier"
						/></Parameter>
						<Parameter label="InputShuffle">
							<InputGroup
								><InputNumber
									v-model="shuffled"
									:min="0"
									:max="1" /><InputShuffle
									v-model="shuffled"
									:generate="shuffleNumber"
							/></InputGroup>
						</Parameter>
					</ParameterGrid>
				</section>

				<section class="reference-section">
					<h2>InputComplex</h2>
					<InputComplex
						v-model="complex"
						:scheme="complexScheme"
						title="Layer"
					/>
				</section>
			</main>
		</Viewport>
	</TweeqProvider>
</template>

<style lang="stylus">
html, body, #app
	min-height 100%

body
	background var(--tq-color-background)

.reference-viewport
	min-height 100vh
	background var(--tq-color-background)

.reference-page
	width calc(100vw - 48px)
	max-width 1080px
	margin 0 auto
	padding 36px 0 72px
	display grid
	gap var(--tq-gap-section)

.page-header
	display flex
	align-items baseline
	gap var(--tq-gap-section)

	h1
		font-size 20px
		font-weight 600

	p
		color var(--tq-color-text-mute)

.reference-section
	border 1px solid var(--tq-color-border)
	border-radius var(--tq-radius-pane)
	background var(--tq-color-surface)
	padding var(--tq-pane-padding)
	display grid
	gap var(--tq-gap-control)

	h2
		font-size 13px
		font-weight 600
		color var(--tq-color-text-mute)

.number-layout
	display grid
	grid-template-columns minmax(240px, 1fr) minmax(360px, 1.4fr)
	gap var(--tq-gap-section)

.subject-card
	display grid
	align-content start
	gap var(--tq-gap-control)
	padding var(--tq-pane-padding)
	border-radius var(--tq-radius-input)
	background var(--tq-color-background)

	> label, output
		color var(--tq-color-text-mute)

	output
		font-family var(--tq-font-code)

.props-grid
	align-content start

.flag-group
	flex-wrap wrap

.state-strip, .component-grid
	display grid
	grid-template-columns repeat(auto-fit, minmax(160px, 1fr))
	gap var(--tq-gap-control)

	> div
		display grid
		align-content start
		gap var(--tq-gap-related)

		> label
			color var(--tq-color-text-mute)

.wide-control
	grid-column span 2

.angle-layout
	display grid
	grid-template-columns 260px 96px minmax(320px, 1fr)
	align-items start
	gap var(--tq-gap-section)

.angle-wide, .angle-compact
	display grid
	gap var(--tq-gap-related)

	> label
		color var(--tq-color-text-mute)

@media (max-width: 760px)
	.reference-page
		width calc(100vw - 24px)
		padding-top 18px

	.number-layout, .angle-layout
		grid-template-columns 1fr

	.page-header
		display grid
		gap var(--tq-gap-related)
</style>
