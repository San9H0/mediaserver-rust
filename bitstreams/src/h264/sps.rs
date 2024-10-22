use crate::h264::errors::H264Error;
use crate::h264::nal_unit::NalUnit;
use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct SPS {
    pub payload: Bytes,
    pub profile_idc: u8,
    pub constraint_compatibility_flag: u8,
    pub level_idc: u8,
    seq_parameter_set_id: u8,
    log2_max_frame_num_minus4: u8,
    pic_order_cnt_type: u8,
    log2_max_pic_order_cnt_lsb_minus4: u8,
    delta_pic_order_always_zero_flag: u8,
    offset_for_non_ref_pic: u8,
    offset_for_top_to_bottom_field: u8,
    num_ref_frames_in_pic_order_cnt_cycle: u8,
    offset_for_ref_frames: Vec<u32>,
    num_ref_frames: u8,
    gaps_in_frame_num_value_allowed_flag: u8,
    pic_width_in_mbs_minus1: u8,
    pic_height_in_map_units_minus1: u8,
    frame_mbs_only_flag: u8,
    mb_adaptive_frame_field_flag: u8,
    direct_8x8_inference_flag: u8,
    frame_cropping_flag: u8,
    frame_crop_left_offset: u8,
    frame_crop_right_offset: u8,
    frame_crop_top_offset: u8,
    frame_crop_bottom_offset: u8,
    vui_parameters_present_flag: u8,
}

impl SPS {
    pub fn from(nalu: &mut NalUnit) -> anyhow::Result<SPS> {
        if nalu.nal_unit_type != 7 {
            anyhow::bail!("Invalid NAL unit type for SPS: {}", nalu.nal_unit_type);
        }

        let mut reader = &mut nalu.reader;
        let profile_idc = reader.read_bits(8)?;
        let constraint_compantion_flag = reader.read_bits(8)?;
        let level_idc = reader.read_bits(8)?;

        let seq_parameter_set_id = reader.read_ue()?;

        let log2_max_frame_num_minus4 = reader.read_ue()?;
        let pic_order_cnt_type = reader.read_ue()?;
        let mut log2_max_pic_order_cnt_lsb_minus4 = 0;
        let mut delta_pic_order_always_zero_flag = 0;
        let mut offset_for_non_ref_pic = 0;
        let mut offset_for_top_to_bottom_field = 0;
        let mut num_ref_frames_in_pic_order_cnt_cycle = 0;
        let mut offset_for_ref_frames = Vec::<u32>::new();
        let mut mb_adaptive_frame_field_flag = 0;
        let mut frame_crop_left_offset = 0;
        let mut frame_crop_right_offset = 0;
        let mut frame_crop_top_offset = 0;
        let mut frame_crop_bottom_offset = 0;

        if pic_order_cnt_type == 0 {
            log2_max_pic_order_cnt_lsb_minus4 = reader.read_ue()?;
        } else if pic_order_cnt_type == 1 {
            delta_pic_order_always_zero_flag = reader.read_bits(1)?;
            offset_for_non_ref_pic = reader.read_se()?;
            offset_for_top_to_bottom_field = reader.read_se()?;
            num_ref_frames_in_pic_order_cnt_cycle = reader.read_ue()?;
            for _ in 0..num_ref_frames_in_pic_order_cnt_cycle {
                let offset_for_ref_frame = reader.read_se()?;
                offset_for_ref_frames.push(offset_for_ref_frame);
            }
        }
        let num_ref_frames = reader.read_ue()?;
        let gaps_in_frame_num_value_allowed_flag = reader.read_bits(1)?;
        let pic_width_in_mbs_minus1 = reader.read_ue()?;
        let pic_height_in_map_units_minus1 = reader.read_ue()?;
        let frame_mbs_only_flag = reader.read_bits(1)?;
        if frame_mbs_only_flag == 0 {
            mb_adaptive_frame_field_flag = reader.read_bits(1)?;
        }
        let direct_8x8_inference_flag = reader.read_bits(1)?;
        let frame_cropping_flag = reader.read_bits(1)?;
        if frame_cropping_flag == 1 {
            frame_crop_left_offset = reader.read_ue()?;
            frame_crop_right_offset = reader.read_ue()?;
            frame_crop_top_offset = reader.read_ue()?;
            frame_crop_bottom_offset = reader.read_ue()?;
        }
        let vui_parameters_present_flag = reader.read_bits(1)?;
        if vui_parameters_present_flag != 0 {
            // todo vui parameters
        }
        //todo rbsp_tailing_bits()

        Ok(SPS {
            payload: nalu.to_bytes(),
            profile_idc,
            constraint_compatibility_flag: constraint_compantion_flag,
            level_idc,
            seq_parameter_set_id,
            log2_max_frame_num_minus4,
            pic_order_cnt_type,
            log2_max_pic_order_cnt_lsb_minus4,
            delta_pic_order_always_zero_flag,
            offset_for_non_ref_pic,
            offset_for_top_to_bottom_field,
            num_ref_frames_in_pic_order_cnt_cycle,
            offset_for_ref_frames,
            num_ref_frames,
            gaps_in_frame_num_value_allowed_flag,
            pic_width_in_mbs_minus1,
            pic_height_in_map_units_minus1,
            frame_mbs_only_flag,
            mb_adaptive_frame_field_flag,
            direct_8x8_inference_flag,
            frame_cropping_flag,
            frame_crop_left_offset,
            frame_crop_right_offset,
            frame_crop_top_offset,
            frame_crop_bottom_offset,
            vui_parameters_present_flag,
        })
    }

    pub fn width(&self) -> u32 {
        (self.pic_width_in_mbs_minus1 + 1) as u32 * 16
    }

    pub fn height(&self) -> u32 {
        (self.pic_height_in_map_units_minus1 + 1) as u32 * 16
    }
}
