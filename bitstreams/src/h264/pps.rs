use crate::h264::nal_unit::NalUnit;
use bytes::Bytes;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PPS {
    pub payload: Bytes,
    pic_parameter_set_id: u8,
    seq_parameter_set_id: u8,
    entropy_coding_mode_flag: u8,
    pic_order_present_flag: u8,
    num_slice_groups_minus1: u8,
    slice_group_map_type: u8,
    num_length_minus1: Vec<u8>,
    top_left: Vec<u8>,
    bottom_right: Vec<u8>,
    slice_group_change_direction_flag: u8,
    slice_group_change_rate_minus1: u8,
    pic_size_in_map_units_minus1: u8,
    slice_group_id: Vec<u8>,
    num_ref_idx_10_active_minus1: u8,
    num_ref_idx_11_active_minus1: u8,
    weighted_pred_flag: u8,
    weighted_bipred_idc: u8,
    pic_init_qp_minus26: u8,
    pic_init_qs_minus26: u8,
    chroma_qp_index_offset: u8,
    deblocking_filter_control_present_flag: u8,
    constrained_intra_pred_flag: u8,
    redundant_pic_cnt_present_flag: u8,
}

impl PPS {
    pub fn from(nalu: &mut NalUnit) -> anyhow::Result<PPS> {
        if nalu.nal_unit_type != 8 {
            anyhow::bail!("Invalid NAL unit type for PPS: {}", nalu.nal_unit_type);
        }

        let pic_parameter_set_id = nalu.reader.read_ue()?;
        let seq_parameter_set_id = nalu.reader.read_ue()?;
        let entropy_coding_mode_flag = nalu.reader.read_bits(1)?;
        let pic_order_present_flag = nalu.reader.read_bits(1)?;
        let num_slice_groups_minus1 = nalu.reader.read_ue()?;

        let mut slice_group_map_type = 0;
        let mut num_length_minus1 = Vec::<u8>::new();
        let mut top_left = Vec::<u8>::new();
        let mut bottom_right = Vec::<u8>::new();
        let mut slice_group_change_direction_flag = 0;
        let mut slice_group_change_rate_minus1 = 0;
        let mut pic_size_in_map_units_minus1 = 0;
        let mut slice_group_id = Vec::<u8>::new();

        if num_slice_groups_minus1 > 0 {
            slice_group_map_type = nalu.reader.read_ue()?;
            if slice_group_map_type == 0 {
                for _ in 0..num_slice_groups_minus1 {
                    num_length_minus1.push(nalu.reader.read_ue()?);
                }
            } else if slice_group_map_type == 2 {
                for _ in 0..num_slice_groups_minus1 {
                    top_left.push(nalu.reader.read_ue()?);
                    bottom_right.push(nalu.reader.read_ue()?);
                }
            } else if slice_group_map_type == 3
                || slice_group_map_type == 4
                || slice_group_map_type == 5
            {
                slice_group_change_direction_flag = nalu.reader.read_bits(1)?;
                slice_group_change_rate_minus1 = nalu.reader.read_ue()?;
            } else if slice_group_map_type == 6 {
                pic_size_in_map_units_minus1 = nalu.reader.read_ue()?;
                for _ in 0..pic_size_in_map_units_minus1 {
                    slice_group_id.push(nalu.reader.read_bits(1)?);
                }
            }
            // rbsp_trailing_bits()
        }
        let num_ref_idx_10_active_minus1 = nalu.reader.read_ue()?;
        let num_ref_idx_11_active_minus1 = nalu.reader.read_ue()?;
        let weighted_pred_flag = nalu.reader.read_bits(1)?;
        let weighted_bipred_idc = nalu.reader.read_bits(2)?;
        let pic_init_qp_minus26 = nalu.reader.read_se()?;
        let pic_init_qs_minus26 = nalu.reader.read_se()?;
        let chroma_qp_index_offset = nalu.reader.read_se()?;
        let deblocking_filter_control_present_flag = nalu.reader.read_bits(1)?;
        let constrained_intra_pred_flag = nalu.reader.read_bits(1)?;
        let redundant_pic_cnt_present_flag = nalu.reader.read_bits(1)?;

        Ok(PPS {
            payload: nalu.to_bytes(),
            pic_parameter_set_id,
            seq_parameter_set_id,
            entropy_coding_mode_flag,
            pic_order_present_flag,
            num_slice_groups_minus1,
            slice_group_map_type,
            num_length_minus1,
            top_left,
            bottom_right,
            slice_group_change_direction_flag,
            slice_group_change_rate_minus1,
            pic_size_in_map_units_minus1,
            slice_group_id,
            num_ref_idx_10_active_minus1,
            num_ref_idx_11_active_minus1,
            weighted_pred_flag,
            weighted_bipred_idc,
            pic_init_qp_minus26,
            pic_init_qs_minus26,
            chroma_qp_index_offset,
            deblocking_filter_control_present_flag,
            constrained_intra_pred_flag,
            redundant_pic_cnt_present_flag,
        })
    }
}
