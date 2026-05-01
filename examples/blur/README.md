# Blur

This example showcases the handling of sRGB colors while blurring. In `gaussian.wgsl`, colors are transformed to linear space before being aggregated. At the end, the sum it converted back to sRGB and stored in the output texture. This causes the gaussian blur to be physically correct, and have a much nicer appearance.

## Example

<div align="center">
<table>
<tr>
<td align="center"><img src="docs/incorrect_blur.png" width="400" /><br /><em>Incorrectly applied blur</em></td>
<td align="center"><img src="docs/correct_blur.png" width="400" /><br /><em>Correctly applied blur</em></td>
</tr>
</table>
</div>